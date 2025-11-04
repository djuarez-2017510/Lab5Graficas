// Vertex struct no usado directamente, mantenido por compatibilidad
#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
}

pub fn load_sphere_or_generate() -> Result<(Vec<tobj::Model>, Vec<tobj::Material>, bool), String> {
    // Intentar varias rutas comunes según el working dir
    let candidates = [
        "sphere.obj",
        "./sphere.obj",
        "./assets/sphere.obj",
        "../sphere.obj",
        "../assets/sphere.obj",
    ];

    for path in &candidates {
        match tobj::load_obj(
            path,
            &tobj::LoadOptions {
                single_index: true,
                triangulate: true,
                ..Default::default()
            },
        ) {
            Ok((models, materials)) => {
                let materials = materials.unwrap_or_else(|_| Vec::new());
                println!("Modelo cargado desde: {}", path);
                log_mesh_info(&models);
                return Ok((models, materials, false)); // false = no fallback
            }
            Err(_) => { /* probar siguiente */ }
        }
    }

    // Si no se encontró: generar UV sphere procedimental
    eprintln!("No se encontró sphere.obj en rutas conocidas. Generando procedimental...");
    let (models, materials) = generate_uv_sphere_models(64, 64, 1.0);
    log_mesh_info(&models);
    Ok((models, materials, true)) // true = usó fallback
}

fn log_mesh_info(models: &Vec<tobj::Model>) {
    println!("Número de meshes: {}", models.len());
    for (i, model) in models.iter().enumerate() {
        let mesh = &model.mesh;
        println!("\n--- Mesh {}: {} ---", i, model.name);
        println!("  Vértices: {}", mesh.positions.len() / 3);
        println!("  Índices: {}", mesh.indices.len());
        println!("  Triángulos: {}", mesh.indices.len() / 3);

        println!("\n  Primeros 5 vértices:");
        for j in 0..5.min(mesh.positions.len() / 3) {
            let x = mesh.positions[j * 3];
            let y = mesh.positions[j * 3 + 1];
            let z = mesh.positions[j * 3 + 2];
            println!("    Vértice {}: ({:.3}, {:.3}, {:.3})", j, x, y, z);
        }
    }
}

// Genera una esfera UV y la empaqueta en tobj::Model para no tocar el resto del pipeline
fn generate_uv_sphere_models(segments: u32, rings: u32, radius: f32) -> (Vec<tobj::Model>, Vec<tobj::Material>) {
    let (positions, indices) = generate_uv_sphere_data(segments, rings, radius);

    let mesh = tobj::Mesh {
        positions,
        indices,
        ..Default::default()
    };

    let model = tobj::Model {
        name: "procedural_uv_sphere".to_string(),
        mesh,
    };

    (vec![model], Vec::new())
}

// Devuelve posiciones & indices de una esfera UV triangulada
fn generate_uv_sphere_data(segments: u32, rings: u32, radius: f32) -> (Vec<f32>, Vec<u32>) {
    let seg = segments.max(3);
    let ring = rings.max(2);

    let mut positions: Vec<f32> = Vec::with_capacity(((ring + 1) * (seg + 1) * 3) as usize);
    let mut indices: Vec<u32> = Vec::with_capacity((ring * seg * 6) as usize);

    for y in 0..=ring {
        let v = y as f32 / ring as f32;
        let theta = v * std::f32::consts::PI; // 0..PI

        for x in 0..=seg {
            let u = x as f32 / seg as f32;
            let phi = u * std::f32::consts::TAU; // 0..2PI

            let px = radius * phi.cos() * theta.sin();
            let py = radius * theta.cos();
            let pz = radius * phi.sin() * theta.sin();

            positions.extend_from_slice(&[px, py, pz]);
        }
    }

    let stride = seg + 1;
    for y in 0..ring {
        for x in 0..seg {
            let i0 = y * stride + x;
            let i1 = i0 + 1;
            let i2 = i0 + stride;
            let i3 = i2 + 1;

            // Dos triángulos por quad
            indices.extend_from_slice(&[i0 as u32, i2 as u32, i1 as u32]);
            indices.extend_from_slice(&[i1 as u32, i2 as u32, i3 as u32]);
        }
    }

    (positions, indices)
}
