use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use macroquad::{prelude::*, rand::gen_range};

use crate::{hero::Hero, particle::Particle};
use crate::light::Light;
use crate::ghost::Ghost;


#[derive(Eq, PartialEq, Hash)]
enum TextureName{
    Background,
    HealthDeco,
    HealthBar,
    Ground,
    Ghost,
    ParticleOne,
    Hero,
    Light,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum TransitionName {
    None,
    FadeIn,
    FadeOut
}

enum GameState {
    Intro,
    Game,
    Win,
    End
}

pub struct Game {
    state: GameState,
    texture_library: HashMap<TextureName, Texture2D>,
    particles: Vec<Particle>,
    max_monsters: i32,
    monster_timer: i32,
    monsters: Vec<Ghost>,
    colliders: Vec<Rect>,
    lights: [Light; 6],
    hero: Hero,

    transition_alpha: f32,
    transition: TransitionName,
    transition_finished: bool
}

impl Game {
    pub fn new() -> Self {

        rand::srand(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64);

        let state = GameState::Intro;
        let background_texture = Texture2D::from_file_with_format(include_bytes!("../assets/sprites/Level.png"), None);
        background_texture.set_filter(FilterMode::Nearest);
        
        let ground_texture = Texture2D::from_file_with_format(include_bytes!("../assets/sprites/Ground.png"), None);
        ground_texture.set_filter(FilterMode::Nearest);
        
        let hero_texture = Texture2D::from_file_with_format(include_bytes!("../assets/sprites/Hero.png"), None);
        hero_texture.set_filter(FilterMode::Nearest);
        
        let particle_one_texture = Texture2D::from_file_with_format(include_bytes!("../assets/sprites/ParticleOne.png"), None);
        particle_one_texture.set_filter(FilterMode::Nearest);

        let light_texture = Texture2D::from_file_with_format(include_bytes!("../assets/sprites/Light.png"), None);
        light_texture.set_filter(FilterMode::Nearest);
        
        let ghost_texture = Texture2D::from_file_with_format(include_bytes!("../assets/sprites/MonsterOne.png"), None);
        ghost_texture.set_filter(FilterMode::Nearest);
       
        let health_container_texture = Texture2D::from_file_with_format(include_bytes!("../assets/sprites/Health_deco.png"), None);
        health_container_texture.set_filter(FilterMode::Nearest);
        
        let health_bar_texture = Texture2D::from_file_with_format(include_bytes!("../assets/sprites/Health_bar.png"), None);
        health_bar_texture.set_filter(FilterMode::Nearest);
        
        let texture_library: HashMap<TextureName, Texture2D> = HashMap::from([
            (TextureName::Background, background_texture),
            (TextureName::Ground, ground_texture),
            (TextureName::Hero, hero_texture),
            (TextureName::ParticleOne, particle_one_texture),
            (TextureName::Light, light_texture),
            (TextureName::Ghost, ghost_texture),
            (TextureName::HealthDeco, health_container_texture),
            (TextureName::HealthBar, health_bar_texture),
        ]);

        let mut particles = Vec::new();
        for _i in 0..100 {
            let part = Particle::new(gen_range(0.0, 426.0), gen_range(0.0, 100.0));
            particles.push(part);
        }

        let lights = [
            Light::new(118.0, 70.0, 32.0),
            Light::new(117.0, 69.0, 24.0),
            Light::new(119.0, 70.0, 30.0),
            Light::new(329.0, 70.0, 32.0),
            Light::new(328.0, 69.0, 24.0),
            Light::new(330.0, 70.0, 30.0),
        ];

        let max_monsters = 5;

        // Delay for the first birth
        let monster_timer = 5;
        // Create empty vec for monster
        let monsters = Vec::new();


        // Level collider (Ground, left and right wall)
        let colliders = vec![
            Rect{x: 0.0, y: 101.0, w: 426.0, h: 16.0},      // Ground
            Rect{x: -16.0, y: 0.0, w: 16.0, h: 112.0},      // Left border
            Rect{x: 426.0, y: 0.0, w: 16.0, h: 112.0},      // Right border

        ];


        Self {
            state,
            texture_library,
            hero: Hero::new(0.0, 0.0),
            particles,
            lights,
            max_monsters,
            colliders,
            monster_timer,
            monsters,

            transition_alpha: 1.0,
            transition: TransitionName::FadeIn,
            transition_finished: false,
        }

    }
    
    pub fn update(&mut self) {
        match self.state {
            GameState::Intro => {
                self.update_decoration();
                if self.transition_finished && self.transition == TransitionName::FadeOut{
                    self.state = GameState::Game;
                    self.transition = TransitionName::FadeIn;
                }
                if is_key_pressed(KeyCode::Space) {
                    self.transition = TransitionName::FadeOut;
                }


               
            },
            GameState::Game => {
                self.monster_timer -= 1;
                if self.max_monsters > 0 && self.monster_timer == 0{
                    self.monster_incubator();
                    self.max_monsters -= 1;
                    self.monster_timer = 20 + gen_range(30, 60);
                }
                // Clean the monster list and remove all dead monster
                self.monsters.retain(|m| m.is_active());


                self.hero.update(&mut self.monsters, &self.colliders);

                for monster in self.monsters.iter_mut() {
                    monster.update(self.hero.position);
                }

                self.update_decoration();

            },
            GameState::End => {},
            GameState::Win => {}
            _ => {},
        }


        // Transition screen update
        match self.transition {
            TransitionName::FadeIn => {
                self.transition_finished = false;
                self.transition_alpha -= 0.01;
                if self.transition_alpha < 0.0 {
                    self.transition_alpha = 0.0;
                    self.transition_finished = true;
                }


            }
            TransitionName::FadeOut => {
                self.transition_finished = false;
                self.transition_alpha += 0.01;
                if self.transition_alpha > 1.0 {
                    self.transition_finished = true;
                    self.transition_alpha = 1.0;
                }
            }
            TransitionName::None => {}
        }
    }

    fn update_decoration(&mut self) {
        for part in self.particles.iter_mut() {
            part.update();
        }

        for light in self.lights.iter_mut() {
            light.update();
        }
    }


    pub fn render(&mut self) {
        clear_background(BLACK);
        self.set_camera_view();

        self.render_background();

        match self.state {
            GameState::Intro => {
                self.render_particles();
                self.render_ground_mask();
                self.render_letterbox_mask();
            },
            GameState::Game => {
                // The hero and thes monsters
                for monster in self.monsters.iter_mut() {
                    let texture = self.texture_library.get(&TextureName::Ghost).expect("No texture in library").clone();
                    monster.sprite.draw_sprite(texture, Vec2::ZERO, 1.0);
                }

                self.hero.sprite.draw_sprite(self.get_texture(TextureName::Hero), Vec2::ZERO, 1.0);

                self.render_ground_mask();
                self.render_particles();
                self.render_letterbox_mask();
                self.render_health_bar();
            },
            GameState::End => {

                self.render_particles();
                self.render_ground_mask();
                self.render_letterbox_mask();
                if is_key_pressed(KeyCode::Space) {
                    self.state = GameState::Game;
                }
            },
            GameState::Win => {

                self.render_particles();
                self.render_ground_mask();
                self.render_letterbox_mask();
                if is_key_pressed(KeyCode::Space) {
                    self.state = GameState::Game;
                }
            },
        }

        // Transition screen
        let color = Color { r: 0.0, g: 0.0, b: 0.0, a: self.transition_alpha };
        draw_rectangle(0.0, -64.0, 426.0, 240.0, color);

        self.debug_info();

    }

    fn render_health_bar(&mut self) {
        // And the health bar decoration
        draw_texture(self.get_texture(TextureName::HealthDeco), 81.0, -48.0, WHITE);
        // Health bar
        draw_texture(self.get_texture(TextureName::HealthBar), 85.0, -36.0, WHITE);
    }

    fn render_letterbox_mask(&mut self) {
        // Letterbox mask (to avoid some artifact)
        draw_rectangle(0.0, -64.0, 426.0, 64.0, BLACK);
        draw_rectangle(0.0, 112.0, 426.0, 64.0, BLACK);
    }

    fn render_particles(&mut self) {
        // Some atmospheric particles
        for part in self.particles.iter_mut() {
            let texture = self.texture_library.get(&TextureName::ParticleOne).expect("No texture in library").clone();
            part.sprite.draw_sprite(texture, Vec2::ZERO, 1.0);
        }
    }

    fn render_ground_mask(&mut self) {
        // The ground to hide some lights
        let bg_params = DrawTextureParams {
            dest_size: Some(Vec2::new(426.0, 112.0)),
            source: Some(Rect::new(0.0, 0.0, 426.0, 112.0)),
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            pivot: None};
        draw_texture_ex(self.get_texture(TextureName::Ground), 0.0 , 0.0, WHITE, bg_params);

    }
    fn render_background(&mut self) {
        let bg_params = DrawTextureParams {
            dest_size: Some(Vec2::new(426.0, 112.0)),
            source: Some(Rect::new(0.0, 0.0, 426.0, 112.0)),
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            pivot: None};
        draw_texture_ex(self.get_texture(TextureName::Background), 0.0 , 0.0, WHITE, bg_params);


        // draw the light
        for light in self.lights.iter() {
            let texture = self.texture_library.get(&TextureName::Light).expect("No texture in library").clone();
            let radius = light.get_radius();
            let params = DrawTextureParams {
                dest_size: Some(Vec2 { x: 2.0 * radius, y: 2.0 * radius }),
                source: Some(Rect{x: 0.0, y: 0.0, w: 64.0, h:64.0}),
                rotation: 0.0,
                flip_x: false,
                flip_y: false,
                pivot: None
            };
            draw_texture_ex(texture, light.get_position().x, light.get_position().y, light.color, params);

        }
    }


    fn get_texture(&self, name: TextureName) -> Texture2D {
        self.texture_library.get(&name).expect("No texture in library").clone()
    }
    
    fn reset_game(&mut self) {
        self.max_monsters = 5;
        self.monsters = Vec::new();
        self.monster_timer = 5;
        self.hero = Hero::new(0.0, 0.0);
        self.state = GameState::Intro;
    }

    fn monster_incubator(&mut self) {
        let m = Ghost::new(gen_range(50.0, 380.0), 52.0);
        self.monsters.push(m);
    }

    /// An ugly experimental empiric camera setting function
    fn set_camera_view(&mut self)  {
        let ratio =  screen_width() / 1278.;
        let h = 240.0 * screen_height() / 720. / ratio;
        let camera = Camera2D::from_display_rect(Rect{x: 0.0, y: -0.5 * (h - 112.0), w: 426.0, h});
        set_camera(&camera);
    }
 
    fn debug_info(&mut self) {
        // Reset game
        if is_key_pressed(KeyCode::Tab) {self.reset_game()}

        // debug rendering
        let h_box = self.hero.get_collision_box(0.0, 0.0);
        draw_rectangle_lines(h_box.x , h_box.y, h_box.w , h_box.h , 1.0, RED);

        // Hero hitbox
        self.hero.debug_hitbox();

        for m in self.monsters.iter() {
            let m_box = m.get_collision_box(0.0, 0.0);
            if m.is_hitable() {

                draw_rectangle_lines(m_box.x , m_box.y, m_box.w , m_box.h , 1.0, RED);
            }
            else {
                draw_rectangle_lines(m_box.x , m_box.y, m_box.w , m_box.h , 1.0, GREEN);

            }
        }
        
        //set_default_camera();    
        //draw_text(&format!("position: {} / {}", self.hero.position.x, self.hero.position.y), 16.0, 32.0, 24.0, RED);

    } 
}