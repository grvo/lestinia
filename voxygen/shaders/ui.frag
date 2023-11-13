#version 330 core

in vec3 f_uv;
in vec4 f_color;

flat in uint f_mode;

uniform sampler2D u_tex;

out vec4 tgt_color;

void main() {
	// texto
	if (f_mode == uint(0)) {
		tgt_color = f_color * vec4(1.0, 1.0, 1.0, texture(u_tex, f_uv).a);

	// imagem
	} else if (f_mode == uint(1)) {
		tgt_color = texture(u_tex, f_uv);

	// geometria 2d
	} else if (f_mode == uint(2)) {
		tgt_color = f_color;
	}
}
