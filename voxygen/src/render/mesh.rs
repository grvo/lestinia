// local
use super::Pipeline;

/// um estrutura de mesh de tipo `vec` utilizado para armazenar dados de mesh na cpu
pub struct Mesh<P: Pipeline> {
    verts: Vec<P::Vertex>
}

impl<P: Pipeline> Mesh<P> {
    /// criar um novo `mesh`
    pub fn new() -> Self {
        Self { verts: vec![] }
    }

    /// obter fatia referenciando os vértices desta malha
    pub fn vertices(&self) -> &[P::Vertex] {
        &self.verts
    }

    /// puxar novo vertex para o final desta malha
    pub fn push(&mut self, vert: P::Vertex) {
        self.verts.push(vert);
    }

    /// puxar novo polígono para o final desta malha
    pub fn push_tri(&mut self, tri: Tri<P>) {
        self.verts.push(tri.a);
        self.verts.push(tri.b);
        self.verts.push(tri.c);
    }

    /// empurrar novo quad para o final desta malha.
    pub fn push_quad(&mut self, quad: Quad<P>) {
        // um quad é composto por dois triângulos

        // tri 1
        self.verts.push(quad.a.clone());
        self.verts.push(quad.b);
        self.verts.push(quad.c.clone());

        // tri 2
        self.verts.push(quad.c);
        self.verts.push(quad.d);
        self.verts.push(quad.a);
    }
}

/// representa um triângulo armazenado na cpu
pub struct Tri<P: Pipeline> {
    a: P::Vertex,
    b: P::Vertex,
    c: P::Vertex,
}

impl<P: Pipeline> Tri<P> {
    pub fn new(
        a: P::Vertex,
        b: P::Vertex,
        c: P::Vertex,
    ) -> Self {
        Self { a, b, c }
    }
}

/// representa um quad armazenado na cpu
pub struct Quad<P: Pipeline> {
    a: P::Vertex,
    b: P::Vertex,
    c: P::Vertex,
    d: P::Vertex
}

impl<P: Pipeline> Quad<P> {
    pub fn new(
        a: P::Vertex,
        b: P::Vertex,
        c: P::Vertex,
        d: P::Vertex
    ) -> Self {
        Self { a, b, c, d }
    }
}