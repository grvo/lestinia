// padrão
use std::{
    collections::{
        HashMap,
        LinkedList
    },

    sync::mpsc,
    time::Duration
};

// biblioteca
use vek::*;

// projeto
use client::Client;

use common::{
    terrain::TerrainMap,

    volumes::vol_map::VolMapErr,
    vol::SampleVol
};

// caixote
use crate::{
    render::{
        Consts,
        Globals,
        Mesh,
        Model,
        Renderer,

        TerrainPipeline,
        TerrainLocals
    },

    mesh::Meshable
};

struct TerrainChunk {
    // dados de gpu
    model: Model<TerrainPipeline>,
    locals: Consts<TerrainLocals>
}

struct ChunkMetaState {
    pos: Vec3<i32>,
    started_tick: u64,
    active_worker: bool
}

/// um tipo produzido por um trabalhador de threads mesh correspondente à posição e mesh de um chunk
struct MeshWorkerResponse {
    pos: Vec3<i32>,
    mesh: Mesh<TerrainPipeline>,
    started_tick: u64
}

/// função executada por trabalhador de threads dedicado ao meshing de chunk
fn mesh_worker(
    pos: Vec3<i32>,
    started_tick: u64,
    volume: <TerrainMap as SampleVol>::Sample
) -> MeshWorkerResponse {
    MeshWorkerResponse {
        pos,
        mesh: volume.generate_mesh(()),
        started_tick
    }
}

pub struct Terrain {
    chunks: HashMap<Vec3<i32>, TerrainChunk>,

    // o sender e receptor mpsc utilizado para conversar com o trabalhador de threads
    // manter o componente sender
    mesh_send_tmp: mpsc::Sender<MeshWorkerResponse>,
    mesh_recv: mpsc::Receiver<MeshWorkerResponse>,
    mesh_todo: LinkedList<ChunkMetaState>
}

impl Terrain {
    pub fn new() -> Self {
        // cria um novo par mpsc para comunicação com threads trabalhadoras
        let (send, recv) = mpsc::channel();
        
        Self {
            chunks: HashMap::new(),

            mesh_send_tmp: send,
            mesh_recv: recv,
            mesh_todo: LinkedList::new()
        }
    }

    pub fn maintain(&mut self, renderer: &mut Renderer, client: &Client) {
        let current_tick = client.get_tick();

        // adicionar qualquer criação recente ou chunks alterados para a lista de chunks a serem meshados
        for pos in client.state().changes().new_chunks.iter()
            .chain(client.state().changes().changed_chunks.iter())
        {
            // todo: outro problema aqui
            // o que acontece se o bloco no topo de um chunk é modificado?
            // é preciso spawnar um trabalhador mesh para corrigir essa vizinhança
            match self.mesh_todo.iter_mut().find(|todo| todo.pos == *pos) {
                Some(todo) => todo.started_tick = current_tick,

                // o chunk está armazenado, adicionar na lista
                None => self.mesh_todo.push_back(ChunkMetaState {
                    pos: *pos,
                    started_tick: current_tick,
                    active_worker: false
                })
            }
        }

        // remover qualquer modelos dos chunks que foram removidos recentemente
        for pos in &client.state().changes().removed_chunks {
            self.chunks.remove(pos);

            self.mesh_todo.drain_filter(|todo| todo.pos == *pos);
        }

        // clonar o sender para a que a thread consiga enviar para o dado de chunk de volta
        // todo: corrigir isso
        let send = self.mesh_send_tmp.clone();

        self.mesh_todo
            .iter_mut()
            // apenas spawnar trabalhadores para jobs meshing sem o trabalhador ativo atual
            .filter(|todo| !todo.active_worker)
            .for_each(|todo| {
                // encontrar a área desejada do terreno
                let aabb = Aabb {
                    min: todo.pos.map2(TerrainMap::chunk_size(), |e, sz| e * sz as i32 - 1),
                    max: todo.pos.map2(TerrainMap::chunk_size(), |e, sz| (e + 1) * sz as i32 + 1)
                };

                // copiar o dado de chunk desejado para performar o meshing
                // fazer isso para obter um sample do terreno incluindo todos os chunks desejados
                let volume = match client.state().terrain().sample(aabb) {
                    Ok(sample) => sample,

                    // caso esse chunk não exista ainda, manter na lista de todo para ser processado mais tarde
                    Err(VolMapErr::NoSuchChunk) => return,

                    _ => panic!("caso edge não tratado")
                };

                // clonar variadas coisas para assim poder movê-las para a thread
                let send = send.clone();
                let pos = todo.pos;

                // listar a thread trabalhadora
                client.thread_pool().execute(move || {
                    send.send(mesh_worker(pos, current_tick, volume))
                        .expect("falha ao enviar mesh de chunk para a thread principal");
                });
                
                todo.active_worker = true;
            });

        // recebe meshes de chunks por meio das threads trabalhadoras
        // postar elas para a gpu e assim armazená-las
        while let Ok(response) = self.mesh_recv.recv_timeout(Duration::new(0, 0)) {
            match self.mesh_todo.iter().find(|todo| todo.pos == response.pos) {
                // é o mesh desejado, inserir o mais novo modelo finalizado para o modelo de terreno
                Some(todo) if response.started_tick == todo.started_tick => {
                    self.chunks.insert(response.pos, TerrainChunk {
                        model: renderer.create_model(&response.mesh).expect("falha ao postar mesh de chunk para a gpu"),
                        
                        locals: renderer.create_consts(&[TerrainLocals {
                            model_offs: response.pos.map2(TerrainMap::chunk_size(), |e, sz| e as f32 * sz as f32).into_array(),
                        }]).expect("falha ao postar os locals do chunk para a gpu")
                    });
                },

                _ => continue
            }
        }
    }

    pub fn render(&self, renderer: &mut Renderer, globals: &Consts<Globals>) {
        for (_, chunk) in &self.chunks {
            renderer.render_terrain_chunk(
                &chunk.model,
                globals,
                &chunk.locals
            );
        }
    }
}
