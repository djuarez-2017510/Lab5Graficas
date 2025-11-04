mod matriz;
mod sphere;
mod rasterizer;
mod shaders;
mod vector;
mod text;
mod renderer;

use minifb::{Key, Window, WindowOptions};
use rasterizer::Framebuffer;
use vector::Vector3;
use shaders::{PlanetShader, ShaderUniforms, StarShader};
use nalgebra::{Matrix4, Point3};
use renderer::{WIDTH, HEIGHT, generate_stars, render_stars, render_planet};
use text::draw_text;

fn main() {
    println!("\nIniciando Software Renderer...");
    
    // Cargar o generar esfera
    let (models, _materials, used_fallback) = sphere::load_sphere_or_generate()
        .expect("No se pudo cargar ni generar la esfera");

    if used_fallback {
        eprintln!("No se encontró sphere.obj, usando ESFERA PROCEDIMENTAL.");
    } else {
        println!("sphere.obj cargada desde archivo.");
    }

    let mesh = &models[0].mesh;
    
    // Crear ventana con minifb
    let mut window = Window::new(
        "Software Renderer - Planetas",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("No se pudo crear la ventana");

    // Limitar FPS
    window.set_target_fps(60);

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let mut time = 0.0f32;
    let mut rotation_angle: f32 = 0.0;
    let mut camera_distance: f32 = 3.5;
    let mut camera_angle: f32 = 0.0;

    // Generar campo de estrellas
    let stars = generate_stars(500);

    // Instanciar SOLO el shader de la estrella
    let mut star_shader = StarShader::default();

    println!("Todo listo! Presiona ESC para salir.\n");
    println!("Controles: A/Z=freq, S/X=speed, D/C=octaves, F/V=disp, G/B=flare, R=reset\n");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // --- Controles de parametros del StarShader ---
        // Frecuencia: A / Z
        if window.is_key_pressed(Key::A, minifb::KeyRepeat::No) {
            star_shader.freq += 0.2;
            println!("Star freq -> {:.2}", star_shader.freq);
        }
        if window.is_key_pressed(Key::Z, minifb::KeyRepeat::No) {
            star_shader.freq = (star_shader.freq - 0.2).max(0.1);
            println!("Star freq -> {:.2}", star_shader.freq);
        }

        // Speed: S / X
        if window.is_key_pressed(Key::S, minifb::KeyRepeat::No) {
            star_shader.speed += 0.05;
            println!("Star speed -> {:.3}", star_shader.speed);
        }
        if window.is_key_pressed(Key::X, minifb::KeyRepeat::No) {
            star_shader.speed = (star_shader.speed - 0.05).max(0.0);
            println!("Star speed -> {:.3}", star_shader.speed);
        }

        // Octaves: D / C
        if window.is_key_pressed(Key::D, minifb::KeyRepeat::No) {
            star_shader.octaves = (star_shader.octaves + 1).min(10);
            println!("Star octaves -> {}", star_shader.octaves);
        }
        if window.is_key_pressed(Key::C, minifb::KeyRepeat::No) {
            star_shader.octaves = (star_shader.octaves - 1).max(1);
            println!("Star octaves -> {}", star_shader.octaves);
        }

        // Displacement scale: F / V
        if window.is_key_pressed(Key::F, minifb::KeyRepeat::No) {
            star_shader.displacement_scale += 0.01;
            println!("Star disp -> {:.3}", star_shader.displacement_scale);
        }
        if window.is_key_pressed(Key::V, minifb::KeyRepeat::No) {
            star_shader.displacement_scale = (star_shader.displacement_scale - 0.01).max(0.0);
            println!("Star disp -> {:.3}", star_shader.displacement_scale);
        }

        // Flare strength: G / B
        if window.is_key_pressed(Key::G, minifb::KeyRepeat::No) {
            star_shader.flare_strength += 0.05;
            println!("Star flare -> {:.3}", star_shader.flare_strength);
        }
        if window.is_key_pressed(Key::B, minifb::KeyRepeat::No) {
            star_shader.flare_strength = (star_shader.flare_strength - 0.05).max(0.0);
            println!("Star flare -> {:.3}", star_shader.flare_strength);
        }

        // Reset parametros: R
        if window.is_key_pressed(Key::R, minifb::KeyRepeat::No) {
            star_shader = StarShader::default();
            println!("Star params RESET to defaults");
        }

        // Controles de camara con flechas
        if window.is_key_down(Key::Left) {
            camera_angle -= 0.02;
        }
        if window.is_key_down(Key::Right) {
            camera_angle += 0.02;
        }
        if window.is_key_down(Key::Up) {
            camera_distance = (camera_distance - 0.05).max(1.5);
        }
        if window.is_key_down(Key::Down) {
            camera_distance = (camera_distance + 0.05).min(10.0);
        }

        // Update
        time += 0.016; // ~60 FPS
        rotation_angle += 0.005;

        // Clear framebuffer
        framebuffer.clear(0x000000);

        // Renderizar fondo de estrellas
        render_stars(&mut framebuffer, &stars);

        // Matrices de transformación para cámara
        let camera_x = camera_distance * camera_angle.sin();
        let camera_z = camera_distance * camera_angle.cos();
        let eye = Point3::new(camera_x, 1.0, camera_z);
        let target = Point3::new(0.0, 0.0, 0.0);
        let up = nalgebra::Vector3::new(0.0, 1.0, 0.0);
        let view = Matrix4::look_at_rh(&eye, &target, &up);
        
        let aspect = WIDTH as f32 / HEIGHT as f32;
        let fov = std::f32::consts::PI / 3.0;
        let projection = Matrix4::new_perspective(aspect, fov, 0.1, 100.0);

        // Configurar uniforms para shaders
        let camera_pos = Vector3::new(camera_x, 1.0, camera_z);
        let uniforms = ShaderUniforms {
            time,
            light_direction: Vector3::new(1.0, 1.0, 0.5).normalize(),
            camera_position: camera_pos,
        };

        // Usar SOLO el shader de la estrella
        let shader: &dyn PlanetShader = &star_shader;

        // Renderizar la estrella
        let model = Matrix4::from_axis_angle(&nalgebra::Vector3::y_axis(), rotation_angle);
        let mvp = projection * view * model;
        render_planet(&mut framebuffer, mesh, &mvp, shader, &uniforms);

        // Dibujar texto de instrucciones en pantalla
        let text_color = 0xFFFFFF; // Blanco
        let scale = 2;
        
        draw_text(&mut framebuffer, 10, 10, "ESTRELLA / SOL - Shader Procedimental", text_color, scale);
        
        // Instrucciones de controles
        draw_text(&mut framebuffer, 10, 720, "A/Z: FREQ | S/X: SPEED | D/C: OCTAVES", text_color, scale);
        draw_text(&mut framebuffer, 10, 740, "F/V: DISP | G/B: FLARE | R: RESET", text_color, scale);
        draw_text(&mut framebuffer, 10, 760, "FLECHAS: CAMARA | ESC: SALIR", text_color, scale);

        // Mostrar parametros del StarShader en pantalla
        let mut y = 40;
        let s1 = format!("Freq (A/Z): {:.2}", star_shader.freq);
        draw_text(&mut framebuffer, 740, y, s1.as_str(), text_color, 1);
        y += 16;
        let s2 = format!("Speed (S/X): {:.3}", star_shader.speed);
        draw_text(&mut framebuffer, 740, y, s2.as_str(), text_color, 1);
        y += 16;
        let s3 = format!("Octaves (D/C): {}", star_shader.octaves);
        draw_text(&mut framebuffer, 740, y, s3.as_str(), text_color, 1);
        y += 16;
        let s4 = format!("Disp (F/V): {:.3}", star_shader.displacement_scale);
        draw_text(&mut framebuffer, 740, y, s4.as_str(), text_color, 1);
        y += 16;
        let s5 = format!("Flare (G/B): {:.3}", star_shader.flare_strength);
        draw_text(&mut framebuffer, 740, y, s5.as_str(), text_color, 1);
        y += 16;
        draw_text(&mut framebuffer, 740, y, "R: reset", text_color, 1);

        // Mostrar en ventana
        window
            .update_with_buffer(&framebuffer.color_buffer, WIDTH, HEIGHT)
            .expect("Error al actualizar ventana");
    }

    println!("\n¡Adiós!");
}