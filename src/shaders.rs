use crate::vector::Vector3;

#[derive(Debug, Clone, Copy)]
pub struct ShaderColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ShaderColor {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        ShaderColor { r, g, b, a }
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        ShaderColor {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        }
    }
}

pub struct ShaderUniforms {
    pub time: f32,
    pub light_direction: Vector3,
    pub camera_position: Vector3,
}

pub trait PlanetShader {
    fn vertex_shader(&self, position: Vector3, normal: Vector3, uv: (f32, f32), uniforms: &ShaderUniforms) -> (Vector3, Vector3);
    fn fragment_shader(&self, position: Vector3, normal: Vector3, uv: (f32, f32), uniforms: &ShaderUniforms) -> ShaderColor;
}

// ============================================================================
// FUNCIONES AUXILIARES
// ============================================================================

// ============================================================================
// SIMPLEX NOISE (3D) + FBM
// Implementacion de Simplex noise 3D (mas eficiente que Perlin, menos artefactos)
// ============================================================================

// Permutation table para Simplex (256 valores duplicados)
static PERM: [u8; 512] = [
    151,160,137,91,90,15,131,13,201,95,96,53,194,233,7,225,
    140,36,103,30,69,142,8,99,37,240,21,10,23,190,6,148,
    247,120,234,75,0,26,197,62,94,252,219,203,117,35,11,32,
    57,177,33,88,237,149,56,87,174,20,125,136,171,168,68,175,
    74,165,71,134,139,48,27,166,77,146,158,231,83,111,229,122,
    60,211,133,230,220,105,92,41,55,46,245,40,244,102,143,54,
    65,25,63,161,1,216,80,73,209,76,132,187,208,89,18,169,
    200,196,135,130,116,188,159,86,164,100,109,198,173,186,3,64,
    52,217,226,250,124,123,5,202,38,147,118,126,255,82,85,212,
    207,206,59,227,47,16,58,17,182,189,28,42,223,183,170,213,
    119,248,152,2,44,154,163,70,221,153,101,155,167,43,172,9,
    129,22,39,253,19,98,108,110,79,113,224,232,178,185,112,104,
    218,246,97,228,251,34,242,193,238,210,144,12,191,179,162,241,
    81,51,145,235,249,14,239,107,49,192,214,31,181,199,106,157,
    184,84,204,176,115,121,50,45,127,4,150,254,138,236,205,93,
    222,114,67,29,24,72,243,141,128,195,78,66,215,61,156,180,
    // repeat
    151,160,137,91,90,15,131,13,201,95,96,53,194,233,7,225,
    140,36,103,30,69,142,8,99,37,240,21,10,23,190,6,148,
    247,120,234,75,0,26,197,62,94,252,219,203,117,35,11,32,
    57,177,33,88,237,149,56,87,174,20,125,136,171,168,68,175,
    74,165,71,134,139,48,27,166,77,146,158,231,83,111,229,122,
    60,211,133,230,220,105,92,41,55,46,245,40,244,102,143,54,
    65,25,63,161,1,216,80,73,209,76,132,187,208,89,18,169,
    200,196,135,130,116,188,159,86,164,100,109,198,173,186,3,64,
    52,217,226,250,124,123,5,202,38,147,118,126,255,82,85,212,
    207,206,59,227,47,16,58,17,182,189,28,42,223,183,170,213,
    119,248,152,2,44,154,163,70,221,153,101,155,167,43,172,9,
    129,22,39,253,19,98,108,110,79,113,224,232,178,185,112,104,
    218,246,97,228,251,34,242,193,238,210,144,12,191,179,162,241,
    81,51,145,235,249,14,239,107,49,192,214,31,181,199,106,157,
    184,84,204,176,115,121,50,45,127,4,150,254,138,236,205,93,
    222,114,67,29,24,72,243,141,128,195,78,66,215,61,156,180,
];

// Gradientes 3D para Simplex
static GRAD3: [[f32; 3]; 12] = [
    [1.0, 1.0, 0.0], [-1.0, 1.0, 0.0], [1.0, -1.0, 0.0], [-1.0, -1.0, 0.0],
    [1.0, 0.0, 1.0], [-1.0, 0.0, 1.0], [1.0, 0.0, -1.0], [-1.0, 0.0, -1.0],
    [0.0, 1.0, 1.0], [0.0, -1.0, 1.0], [0.0, 1.0, -1.0], [0.0, -1.0, -1.0],
];

fn simplex_noise(x: f32, y: f32, z: f32) -> f32 {
    // Skewing/unskewing factors para 3D
    const F3: f32 = 1.0 / 3.0;
    const G3: f32 = 1.0 / 6.0;

    // Skew input space
    let s = (x + y + z) * F3;
    let i = (x + s).floor();
    let j = (y + s).floor();
    let k = (z + s).floor();

    let t = (i + j + k) * G3;
    let x0_base = i - t;
    let y0_base = j - t;
    let z0_base = k - t;

    let x0 = x - x0_base;
    let y0 = y - y0_base;
    let z0 = z - z0_base;

    // Determinar orden de los simplices
    let (i1, j1, k1, i2, j2, k2) = if x0 >= y0 {
        if y0 >= z0 {
            (1, 0, 0, 1, 1, 0)
        } else if x0 >= z0 {
            (1, 0, 0, 1, 0, 1)
        } else {
            (0, 0, 1, 1, 0, 1)
        }
    } else {
        if y0 < z0 {
            (0, 0, 1, 0, 1, 1)
        } else if x0 < z0 {
            (0, 1, 0, 0, 1, 1)
        } else {
            (0, 1, 0, 1, 1, 0)
        }
    };

    let x1 = x0 - i1 as f32 + G3;
    let y1 = y0 - j1 as f32 + G3;
    let z1 = z0 - k1 as f32 + G3;
    let x2 = x0 - i2 as f32 + 2.0 * G3;
    let y2 = y0 - j2 as f32 + 2.0 * G3;
    let z2 = z0 - k2 as f32 + 2.0 * G3;
    let x3 = x0 - 1.0 + 3.0 * G3;
    let y3 = y0 - 1.0 + 3.0 * G3;
    let z3 = z0 - 1.0 + 3.0 * G3;

    // Hash coordinates
    let ii = i as i32 & 255;
    let jj = j as i32 & 255;
    let kk = k as i32 & 255;

    let gi0 = PERM[(ii + PERM[(jj + PERM[kk as usize] as i32) as usize] as i32) as usize] as usize % 12;
    let gi1 = PERM[(ii + i1 + PERM[(jj + j1 + PERM[(kk + k1) as usize] as i32) as usize] as i32) as usize] as usize % 12;
    let gi2 = PERM[(ii + i2 + PERM[(jj + j2 + PERM[(kk + k2) as usize] as i32) as usize] as i32) as usize] as usize % 12;
    let gi3 = PERM[(ii + 1 + PERM[(jj + 1 + PERM[(kk + 1) as usize] as i32) as usize] as i32) as usize] as usize % 12;

    // Contribuciones de cada esquina
    let mut n0 = 0.0;
    let t0 = 0.6 - x0 * x0 - y0 * y0 - z0 * z0;
    if t0 > 0.0 {
        let t0_sq = t0 * t0;
        n0 = t0_sq * t0_sq * (GRAD3[gi0][0] * x0 + GRAD3[gi0][1] * y0 + GRAD3[gi0][2] * z0);
    }

    let mut n1 = 0.0;
    let t1 = 0.6 - x1 * x1 - y1 * y1 - z1 * z1;
    if t1 > 0.0 {
        let t1_sq = t1 * t1;
        n1 = t1_sq * t1_sq * (GRAD3[gi1][0] * x1 + GRAD3[gi1][1] * y1 + GRAD3[gi1][2] * z1);
    }

    let mut n2 = 0.0;
    let t2 = 0.6 - x2 * x2 - y2 * y2 - z2 * z2;
    if t2 > 0.0 {
        let t2_sq = t2 * t2;
        n2 = t2_sq * t2_sq * (GRAD3[gi2][0] * x2 + GRAD3[gi2][1] * y2 + GRAD3[gi2][2] * z2);
    }

    let mut n3 = 0.0;
    let t3 = 0.6 - x3 * x3 - y3 * y3 - z3 * z3;
    if t3 > 0.0 {
        let t3_sq = t3 * t3;
        n3 = t3_sq * t3_sq * (GRAD3[gi3][0] * x3 + GRAD3[gi3][1] * y3 + GRAD3[gi3][2] * z3);
    }

    // Suma y normaliza a [0,1]
    let result = 32.0 * (n0 + n1 + n2 + n3);
    (result * 0.5) + 0.5
}

fn fbm_simplex(x: f32, y: f32, z: f32, octaves: i32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 0.5;
    let mut frequency = 1.0;

    for _ in 0..octaves {
        value += amplitude * simplex_noise(x * frequency, y * frequency, z * frequency);
        frequency *= 2.0;
        amplitude *= 0.5;
    }

    value
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

fn mix_color(a: ShaderColor, b: ShaderColor, t: f32) -> ShaderColor {
    ShaderColor::new(
        mix(a.r, b.r, t),
        mix(a.g, b.g, t),
        mix(a.b, b.b, t),
        mix(a.a, b.a, t),
    )
}

// ============================================================================
// PLANETA 1: PLANETA ROCOSO CON CRÁTERES (MÁS SUAVE)
// ============================================================================
pub struct RockyPlanetShader;

impl PlanetShader for RockyPlanetShader {
    fn vertex_shader(&self, position: Vector3, normal: Vector3, _uv: (f32, f32), _uniforms: &ShaderUniforms) -> (Vector3, Vector3) {
        // Deformación más pronunciada para cráteres y montañas
        let crater_noise = fbm_simplex(position.x * 4.0, position.y * 4.0, position.z * 4.0, 4);
        let mountain_noise = fbm_simplex(position.x * 2.0, position.y * 2.0, position.z * 2.0, 3);
        let displacement = (crater_noise - 0.5) * 0.15 + (mountain_noise - 0.5) * 0.08; // Más rocoso
        
        let deformed = Vector3::new(
            position.x + normal.x * displacement,
            position.y + normal.y * displacement,
            position.z + normal.z * displacement,
        );
        
        (deformed, normal)
    }

    fn fragment_shader(&self, _position: Vector3, normal: Vector3, uv: (f32, f32), uniforms: &ShaderUniforms) -> ShaderColor {
        // Colores más suaves
        let base_brown = ShaderColor::from_rgb(120, 90, 70);
        let light_brown = ShaderColor::from_rgb(160, 130, 100);
        let dark_brown = ShaderColor::from_rgb(80, 60, 45);
        
        // Textura suave
        let texture = fbm_simplex(uv.0 * 8.0, uv.1 * 8.0, 0.0, 3);
        
        let base_color = if texture > 0.55 {
            mix_color(base_brown, light_brown, smoothstep(0.55, 0.7, texture))
        } else {
            mix_color(dark_brown, base_brown, smoothstep(0.4, 0.55, texture))
        };
        
        // Iluminación
        let light_dir = uniforms.light_direction.normalize();
        let diffuse = normal.dot(&light_dir).max(0.0);
        let ambient = 0.3;
        let lighting = (ambient + diffuse * 0.7).min(1.0);
        
        ShaderColor::new(
            (base_color.r * lighting).clamp(0.0, 1.0),
            (base_color.g * lighting).clamp(0.0, 1.0),
            (base_color.b * lighting).clamp(0.0, 1.0),
            1.0,
        )
    }
}

// ============================================================================
// PLANETA 2: GIGANTE GASEOSO CON ANILLOS
// ============================================================================
pub struct GasGiantShader;

impl PlanetShader for GasGiantShader {
    fn vertex_shader(&self, position: Vector3, normal: Vector3, _uv: (f32, f32), _uniforms: &ShaderUniforms) -> (Vector3, Vector3) {
        (position, normal)
    }

    fn fragment_shader(&self, _position: Vector3, normal: Vector3, uv: (f32, f32), uniforms: &ShaderUniforms) -> ShaderColor {
        // Colores de gigante gaseoso (naranja/crema)
        let orange = ShaderColor::from_rgb(220, 150, 80);
        let cream = ShaderColor::from_rgb(240, 200, 150);
        let dark_orange = ShaderColor::from_rgb(180, 100, 50);
        
        // Bandas horizontales suaves
        let bands = (uv.1 * 12.0 + uniforms.time * 0.1).sin() * 0.5 + 0.5;
        let turbulence = fbm_simplex(uv.0 * 10.0, uv.1 * 5.0, uniforms.time * 0.05, 2) * 0.3;
        
        let color_mix = bands + turbulence;
        let base_color = if color_mix > 0.6 {
            mix_color(orange, cream, smoothstep(0.6, 0.8, color_mix))
        } else {
            mix_color(dark_orange, orange, smoothstep(0.3, 0.6, color_mix))
        };
        
        // Iluminación
        let light_dir = uniforms.light_direction.normalize();
        let diffuse = normal.dot(&light_dir).max(0.0);
        let ambient = 0.35;
        let lighting = (ambient + diffuse * 0.65).min(1.0);
        
        ShaderColor::new(
            (base_color.r * lighting).clamp(0.0, 1.0),
            (base_color.g * lighting).clamp(0.0, 1.0),
            (base_color.b * lighting).clamp(0.0, 1.0),
            1.0,
        )
    }
}

// ============================================================================
// PLANETA 3: PLANETA OCEÁNICO (MÁS SUAVE)
// ============================================================================
pub struct BioLuminescentShader;

impl PlanetShader for BioLuminescentShader {
    fn vertex_shader(&self, position: Vector3, normal: Vector3, _uv: (f32, f32), _uniforms: &ShaderUniforms) -> (Vector3, Vector3) {
        // Sin deformaciones - superficie completamente lisa
        (position, normal)
    }

    fn fragment_shader(&self, position: Vector3, normal: Vector3, uv: (f32, f32), uniforms: &ShaderUniforms) -> ShaderColor {
        // Colores oceánicos suaves
        let deep_blue = ShaderColor::from_rgb(30, 60, 120);
        let ocean_blue = ShaderColor::from_rgb(50, 100, 180);
        let light_blue = ShaderColor::from_rgb(80, 140, 220);
        let foam = ShaderColor::from_rgb(200, 220, 240);
        
        // Patrón oceánico suave
        let ocean_pattern = fbm_simplex(uv.0 * 6.0, uv.1 * 6.0, uniforms.time * 0.1, 3);
        let wave_pattern = (uv.0 * 15.0 + uniforms.time * 0.2).sin() * 0.5 + 0.5;
        
        let combined = ocean_pattern * 0.7 + wave_pattern * 0.3;
        
        let base_color = if combined > 0.7 {
            mix_color(light_blue, foam, smoothstep(0.7, 0.85, combined))
        } else if combined > 0.45 {
            mix_color(ocean_blue, light_blue, smoothstep(0.45, 0.7, combined))
        } else {
            mix_color(deep_blue, ocean_blue, smoothstep(0.2, 0.45, combined))
        };
        
        // Iluminación con brillo oceánico
        let light_dir = uniforms.light_direction.normalize();
        let view_dir = (uniforms.camera_position - position).normalize();
        let diffuse = normal.dot(&light_dir).max(0.0);
        let specular = view_dir.dot(&normal).max(0.0).powf(20.0) * 0.3;
        let ambient = 0.25;
        let lighting = (ambient + diffuse * 0.6 + specular).min(1.2);
        
        ShaderColor::new(
            (base_color.r * lighting).clamp(0.0, 1.0),
            (base_color.g * lighting).clamp(0.0, 1.0),
            (base_color.b * lighting).clamp(0.0, 1.0),
            1.0,
        )
    }
}

// ============================================================================
// PLANETA 4: PLANETA HELADO
// ============================================================================
pub struct IcePlanetShader;

impl PlanetShader for IcePlanetShader {
    fn vertex_shader(&self, position: Vector3, normal: Vector3, _uv: (f32, f32), _uniforms: &ShaderUniforms) -> (Vector3, Vector3) {
        // Sin deformaciones - superficie completamente lisa
        (position, normal)
    }

    fn fragment_shader(&self, position: Vector3, normal: Vector3, uv: (f32, f32), uniforms: &ShaderUniforms) -> ShaderColor {
        // Colores de hielo
        let ice_white = ShaderColor::from_rgb(240, 245, 255);
        let ice_blue = ShaderColor::from_rgb(180, 210, 240);
        let dark_ice = ShaderColor::from_rgb(140, 170, 200);
        
        let ice_pattern = fbm_simplex(uv.0 * 10.0, uv.1 * 10.0, 0.0, 3);
        let cracks = fbm_simplex(uv.0 * 20.0, uv.1 * 20.0, 1.0, 2);
        
        let base_color = if ice_pattern > 0.6 {
            mix_color(ice_blue, ice_white, smoothstep(0.6, 0.8, ice_pattern))
        } else {
            mix_color(dark_ice, ice_blue, smoothstep(0.3, 0.6, ice_pattern))
        };
        
        // Grietas oscuras
        let final_color = if cracks < 0.2 {
            mix_color(base_color, dark_ice, smoothstep(0.0, 0.2, cracks))
        } else {
            base_color
        };
        
        // Iluminación brillante (hielo refleja mucha luz)
        let light_dir = uniforms.light_direction.normalize();
        let view_dir = (uniforms.camera_position - position).normalize();
        let diffuse = normal.dot(&light_dir).max(0.0);
        let specular = view_dir.dot(&normal).max(0.0).powf(30.0) * 0.5;
        let ambient = 0.4;
        let lighting = (ambient + diffuse * 0.5 + specular).min(1.3);
        
        ShaderColor::new(
            (final_color.r * lighting).clamp(0.0, 1.0),
            (final_color.g * lighting).clamp(0.0, 1.0),
            (final_color.b * lighting).clamp(0.0, 1.0),
            1.0,
        )
    }
}

// ============================================================================
// PLANETA 5: PLANETA VOLCÁNICO
// ============================================================================
pub struct VolcanicPlanetShader;

impl PlanetShader for VolcanicPlanetShader {
    fn vertex_shader(&self, position: Vector3, normal: Vector3, _uv: (f32, f32), _uniforms: &ShaderUniforms) -> (Vector3, Vector3) {
        // Sin deformaciones - superficie completamente lisa
        (position, normal)
    }

    fn fragment_shader(&self, _position: Vector3, normal: Vector3, uv: (f32, f32), uniforms: &ShaderUniforms) -> ShaderColor {
        // Colores volcánicos
        let dark_rock = ShaderColor::from_rgb(40, 30, 30);
        let lava_red = ShaderColor::from_rgb(200, 50, 20);
        let lava_orange = ShaderColor::from_rgb(255, 140, 30);
        let lava_yellow = ShaderColor::from_rgb(255, 220, 100);
        
        // Patrón de lava
        let lava_flow = fbm_simplex(uv.0 * 8.0, uv.1 * 8.0, uniforms.time * 0.2, 3);
        let pulse = (uniforms.time * 2.0).sin() * 0.5 + 0.5;
        
        let heat = lava_flow * 0.7 + pulse * 0.3;
        
        let base_color = if heat > 0.7 {
            mix_color(lava_orange, lava_yellow, smoothstep(0.7, 0.85, heat))
        } else if heat > 0.4 {
            mix_color(lava_red, lava_orange, smoothstep(0.4, 0.7, heat))
        } else {
            mix_color(dark_rock, lava_red, smoothstep(0.1, 0.4, heat))
        };
        
        // Iluminación + emisión de lava
        let light_dir = uniforms.light_direction.normalize();
        let diffuse = normal.dot(&light_dir).max(0.0);
        let emission = heat * 0.6; // La lava emite luz
        let ambient = 0.2;
        let lighting = (ambient + diffuse * 0.3 + emission).min(1.5);
        
        ShaderColor::new(
            (base_color.r * lighting).clamp(0.0, 1.0),
            (base_color.g * lighting).clamp(0.0, 1.0),
            (base_color.b * lighting).clamp(0.0, 1.0),
            1.0,
        )
    }
}

// ============================================================================
// ESTRELLA / SOL - Shader procedimental usando Perlin + FBM
// - Animacion continua y ciclica usando `uniforms.time`
// - Emision variable y picos de energia
// - Deformacion en vertex shader para simular flare/distorsion
// ============================================================================
pub struct StarShader {
    pub freq: f32,
    pub speed: f32,
    pub octaves: i32,
    pub displacement_scale: f32,
    pub flare_strength: f32,
}

impl Default for StarShader {
    fn default() -> Self {
        StarShader {
            freq: 3.5,
            speed: 0.35,
            octaves: 6,
            displacement_scale: 0.08,
            flare_strength: 0.35,
        }
    }
}

impl PlanetShader for StarShader {
    fn vertex_shader(&self, position: Vector3, normal: Vector3, _uv: (f32, f32), uniforms: &ShaderUniforms) -> (Vector3, Vector3) {
        // Displacement basado en FBM de Simplex para turbulencias en la superficie
        let noise = fbm_simplex(position.x * self.freq, position.y * self.freq, position.z * self.freq + uniforms.time * self.speed, self.octaves);

        // Centrar alrededor de 0
        let centered = (noise - 0.5) * 2.0;

        // Deformacion base
        let displacement = centered * self.displacement_scale;

        // Flare radial: zonas con noise elevado reciben un empuje mayor
        let flare = (noise - 0.6).max(0.0).powf(2.0) * self.flare_strength;

        let total_disp = displacement + flare;

        let deformed = Vector3::new(
            position.x + normal.x * total_disp,
            position.y + normal.y * total_disp,
            position.z + normal.z * total_disp,
        );

        (deformed, normal)
    }

    fn fragment_shader(&self, position: Vector3, _normal: Vector3, _uv: (f32, f32), uniforms: &ShaderUniforms) -> ShaderColor {
        // Usamos coordinates del espacio 3D para ruido (menos costuras)
        let p = position.normalize();

        // Mapear ruido FBM a intensidad base usando parametros del shader
        let n = fbm_simplex(p.x * (self.freq * 0.6), p.y * (self.freq * 0.6), p.z * (self.freq * 0.6) + uniforms.time * (self.speed * 0.7), self.octaves);

        // Pulso ciclico para que la animacion sea repetible y ciclica
        let pulse = ((uniforms.time * 0.6).sin() * 0.5) + 0.5; // [0,1]

        // Intensidad combinada: ruido + pulso
        let intensity = (n * 0.75 + pulse * 0.25).clamp(0.0, 1.0);

        // Emision variable: picos donde el ruido es alto
        // Base emission ALTA para evitar áreas oscuras/negras
        let base_emission = 0.95; // Emisión mínima muy alta (sin negro)
        let emission = base_emission + smoothstep(0.3, 0.9, intensity) * (0.5 + intensity * 0.8);

        // Gradiente de color: negro -> rojo oscuro -> naranja -> rojo brillante
        let color_core = ShaderColor::from_rgb(10, 5, 0);       // Negro/marrón muy oscuro (núcleo)
        let color_mid = ShaderColor::from_rgb(180, 40, 0);      // Rojo oscuro anaranjado (medio)
        let color_hot = ShaderColor::from_rgb(255, 100, 0);     // Naranja rojizo brillante (caliente)
        let color_peak = ShaderColor::from_rgb(255, 150, 50);   // Naranja brillante (picos)

        // Mix en múltiples etapas para gradiente suave
        let t1 = smoothstep(0.0, 0.35, intensity);
        let t2 = smoothstep(0.35, 0.65, intensity);
        let t3 = smoothstep(0.65, 1.0, intensity);

        let c1 = mix_color(color_core, color_mid, t1);
        let c2 = mix_color(c1, color_hot, t2);
        let final_color = mix_color(c2, color_peak, t3);

        // Aplicar emision al color (la estrella emite luz propia)
        ShaderColor::new(
            (final_color.r * emission).clamp(0.0, 1.0),
            (final_color.g * emission).clamp(0.0, 1.0),
            (final_color.b * emission).clamp(0.0, 1.0),
            1.0,
        )
    }
}

