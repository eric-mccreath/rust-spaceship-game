// rust-spaceship-game - Just a simple asteriod spaceship shooting game written in Rust.
// So left and right arrow keys to spin, up arrow for thrust, and space bar to shoot.
// Eric McCreath GPL-3.0


extern crate piston_window;
extern crate find_folder;

use piston_window::*;
use std::f64::consts::PI;
use rand::Rng;

//  TDVec - a simple 2d vector class.  There is probably something more standard I could use!!
pub struct TDVec {
    x: f64,
    y: f64,
}

impl TDVec {
    fn add(&self, v2: &TDVec) -> TDVec {
        TDVec {
            x: self.x + v2.x,
            y: self.y + v2.y,
        }
    }
    fn wrapppos(&mut self) {
        if self.x < 0.0 { self.x += 640.0; }
        if self.x >= 640.0 { self.x -= 640.0; }
        if self.y < 0.0 { self.y += 480.0; }
        if self.y >= 480.0 { self.y -= 480.0; }
    }

    fn distance(&self, v2: &TDVec) -> f64 {
        let dx = self.x - v2.x;
        let dy = self.y - v2.y;
        (dx * dx + dy * dy).sqrt()
    }
}

// Missile - the missiles you shoot
pub struct Missile {
    pos: TDVec,
    vel: TDVec,
    steps: i32,
}


impl Missile {
    fn update(&mut self) {
        self.pos = self.pos.add(&self.vel);
        self.pos.wrapppos();
        self.steps += 1;
        // self.vel.y += 0.01; // gravity
    }


    fn draw(&mut self, context: &Context, graphics: &mut G2d) {
        let (cx, cy) = (self.pos.x, self.pos.y);
        rectangle([0.1, 0.7, 0.0, 1.0], [cx - 1.0, cy - 1.0, 2.0, 2.0], context.transform, graphics);
    }
}


// Rock - a floating asteroid
pub struct Rock {
    pos: TDVec,
    vel: TDVec,
    r: f64,
}

impl Rock {
    fn update(&mut self) {
        self.pos = self.pos.add(&self.vel);
        self.pos.wrapppos();
    }

    fn is_hit(&self, missiles: &Vec<Missile>) -> bool {
        for m in missiles.iter() {
            if m.pos.distance(&self.pos) < self.r {
                return true;
            }
        }
        return false;
    }

    fn draw(&mut self, context: &Context, graphics: &mut G2d) {
        let (cx, cy) = (self.pos.x, self.pos.y);
        circle_arc([0.1, 0.3, 0.0, 1.0], 0.6, 0.0, 2.0 * PI, [cx - self.r, cy - self.r, 2.0 * self.r, 2.0 * self.r], context.transform, graphics);
        // circle_arc([0.9,0.7,0.0,1.0],2.0, 0.0, 2.0*PI, [cx-self.r, cy-self.r, 2.0 * self.r,2.0 * self.r], context.transform,graphics );
    }
}

// Ship - the space ship.  This class is used for the ship you control and also for drawing the
//        remaining ships.
pub struct Ship {
    pos: TDVec,
    vel: TDVec,
    dir: f64,
    fire: bool,
    thrust_a: f64,
    rotate_a: f64,
}

impl Ship {
    fn update(&mut self) {
        self.pos = self.pos.add(&self.vel);
        self.pos.wrapppos();
        self.vel.x += self.thrust_a * self.dir.sin();
        self.vel.y += self.thrust_a * self.dir.cos();

        self.dir += self.rotate_a;
        if self.dir < 0.0 {
            self.dir += 2.0 * PI;
        }
        if self.dir >= 2.0 * PI {
            self.dir -= 2.0 * PI;
        }
        // self.vel.y += 0.01; // gravity
    }

    fn rotate(&mut self, a: f64) {
        self.rotate_a = a;
    }

    fn thrust(&mut self, p: f64) {
        self.fire = true;
        self.thrust_a = p;
    }

    fn thrustoff(&mut self) {
        self.fire = false;
        self.thrust_a = 0.0;
    }

    fn hit(&mut self, rock: &Rock) -> bool {
        return &rock.pos.distance(&self.pos) < &rock.r;
    }

    fn hits(&mut self, rocks: &Vec<Rock>) -> bool {
        for i in rocks.iter() {
            if self.hit(&i) {
                return true;
            }
        }
        return false;
    }


    fn draw(&mut self, context: &Context, graphics: &mut G2d) {
        let (cx, cy) = (self.pos.x, self.pos.y);
        let lines: [(f64, f64); 4] = [(10.0, 0.0), (6.0, 3.0 * PI / 4.0), (2.0, PI), (6.0, 5.0 * PI / 4.0)];
        for i in 0..4 {
            let (sw, sa) = lines[i];
            let (nw, na) = lines[(i + 1) % 4];
            let sa = sa + self.dir;
            let na = na + self.dir;
            let (sx, sy) = (cx + sw * sa.sin(), cy + sw * sa.cos());
            let (nx, ny) = (cx + nw * na.sin(), cy + nw * na.cos());
            line([0.1, 0.4, 1.0, 1.0], 0.5, [sx, sy, nx, ny], context.transform, graphics);
        }

        if self.fire {
            let lines: [(f64, f64); 4] = [(3.0, PI), (6.0, 7.0 * PI / 8.0), (9.0, PI), (6.0, 9.0 * PI / 8.0)];
            for i in 0..4 {
                let (sw, sa) = lines[i];
                let (nw, na) = lines[(i + 1) % 4];
                let sa = sa + self.dir;
                let na = na + self.dir;
                let (sx, sy) = (cx + sw * sa.sin(), cy + sw * sa.cos());
                let (nx, ny) = (cx + nw * na.sin(), cy + nw * na.cos());
                line([1.0, 0.4, 1.0, 1.0], 0.5, [sx, sy, nx, ny], context.transform, graphics);
            }
        }
    }
}

// Game - this class captures the entire game state.
pub struct Game {
    ship: Ship,
    missiles: Vec<Missile>,
    rocks: Vec<Rock>,
    score: i32,
    shipcount: i32,
}

impl Game {
    fn draw(&mut self, win: &mut PistonWindow, event: Event, glyphs: &mut Glyphs) {
        win.draw_2d(&event, |context, graphics, _device| {
            clear([1.0; 4], graphics);
            self.ship.draw(&context, graphics);
            for m in self.missiles.iter_mut() {
                m.draw(&context, graphics);
            }
            for r in self.rocks.iter_mut() {
                r.draw(&context, graphics);
            }
            for i in 0..self.shipcount {
                let xpos = 500.0 + 15.0 * (i as f64);
                let mut remaining_ship = Ship {
                    pos: TDVec {
                        //x: 10.0 + 20.0*(i as f64),
                        x: xpos,
                        y: 15.0,
                    },
                    vel: TDVec {
                        x: 0.0,
                        y: 0.0,
                    },
                    dir: PI,
                    fire: false,
                    thrust_a: 0.0,
                    rotate_a: 0.0,
                };
                remaining_ship.draw(&context, graphics);
            }
        });


        win.draw_2d(&event, |c, g, device| {
            let transform = c.transform.trans(540.0, 18.0);

            // clear([0.0, 0.0, 0.0, 1.0], g);
            let scoretext = format!("Score: {}", self.score);
            text::Text::new_color([0.2, 0.2, 0.2, 1.0], 12).draw(
                &scoretext,
                glyphs,
                &c.draw_state,
                transform, g,
            ).unwrap();
            glyphs.factory.encoder.flush(device);
        });

        if !self.playing() {
            win.draw_2d(&event, |c, g, device| {
                let transform = c.transform.trans(250.0, 200.0);

                // clear([0.0, 0.0, 0.0, 1.0], g);
                let gameovertext = format!("Game Over");
                text::Text::new_color([1.0, 0.0, 0.0, 1.0], 22).draw(
                    &gameovertext,
                    glyphs,
                    &c.draw_state,
                    transform, g,
                ).unwrap();

                let transform = c.transform.trans(250.0, 230.0);
                let text = format!("Press S to start");
                text::Text::new_color([0.2, 0.0, 1.0, 1.0], 22).draw(
                    &text,
                    glyphs,
                    &c.draw_state,
                    transform, g,
                ).unwrap();

                glyphs.factory.encoder.flush(device);
            });
        }
    }

    fn shoot(&mut self) {
        let a = self.ship.dir;
        let v = 6.0;
        let (vx, vy) = (v * a.sin() + self.ship.vel.x, v * a.cos() + self.ship.vel.y);
        self.missiles.push(Missile {
            pos: TDVec {
                x: self.ship.pos.x,
                y: self.ship.pos.y,
            },
            vel: TDVec {
                x: vx,
                y: vy,
            },
            steps: 0,
        })
    }

    fn playing(&mut self) -> bool {
        return self.shipcount > 0;
    }

    fn update(&mut self) {
        if self.playing() {
            self.ship.update();
            for m in self.missiles.iter_mut() {
                m.update();
            }
            for r in self.rocks.iter_mut() {
                r.update();
            }
            self.missiles.retain(|m| m.steps < 100);

            let mut pos = 0;
            while pos < self.rocks.len() {
                if self.rocks[pos].is_hit(&self.missiles) {
                    self.rocks.swap_remove(pos);
                } else {
                    pos += 1;
                }
            }

            if self.ship.hits(&self.rocks) {
                self.shipcount -= 1;
            }
        }
    }

    fn restart(&mut self) {
        init_game_state(&mut *self);

    }

    fn input(&mut self, inp: &Input) {
        match inp {
            Input::Button(but) => match but.state {
                ButtonState::Press => match but.button {
                    Button::Keyboard(Key::Up) => self.ship.thrust(0.15),
                    Button::Keyboard(Key::Down) => {}
                    Button::Keyboard(Key::Left) => self.ship.rotate(0.1),
                    Button::Keyboard(Key::Right) => self.ship.rotate(-0.1),
                    Button::Keyboard(Key::Space) => self.shoot(),
                    Button::Keyboard(Key::S) => self.restart(),
                    _ => (),
                },
                ButtonState::Release => match but.button {
                    Button::Keyboard(Key::Up) => self.ship.thrustoff(),
                    Button::Keyboard(Key::Down) => {}
                    Button::Keyboard(Key::Left) => self.ship.rotate(0.0),
                    Button::Keyboard(Key::Right) => self.ship.rotate(0.0),
                    _ => (),
                }
            },
            _ => {}
        }
    }
}


fn main() {
    // set up the window
    let mut window: PistonWindow =
        WindowSettings::new("Rust SpaceShip Game", [640, 480])
            .exit_on_esc(true).build().unwrap();
    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    println!("{:?}", assets);

    // FreeMono.otf - was obtained from  http://ftp.gnu.org/gnu/freefont/ and is also GPL v3.
    let mut glyphs = window.load_font(assets.join("FreeMono.otf")).unwrap();
    window.set_lazy(true);

    // create the initial game state
    let mut game = Game {
        ship: Ship {
            pos: TDVec {
                x: 30.0,
                y: 30.0,
            },
            vel: TDVec {
                x: 0.0,
                y: 0.0,
            },
            dir: PI,
            fire: false,
            thrust_a: 0.0,
            rotate_a: 0.0,
        },
        missiles: Vec::new(),
        rocks: Vec::new(),
        score: 0,
        shipcount: 3,
    };
    init_game_state(&mut game);

    // the main event loop
    let mut events = Events::new(EventSettings::new()).ups(30);
    while let Some(event) = events.next(&mut window) {
        match event {
            Event::Loop(Loop::Update(ref _upd)) => game.update(),
            Event::Loop(Loop::Render(ref _ren)) => game.draw(&mut window, event, &mut glyphs),
            Event::Input(ref inp, _) => game.input(inp),
            _ => {}
        }
    }
}

fn init_game_state(game: &mut Game) {
    game.ship = Ship {
        pos: TDVec {
            x: 30.0,
            y: 30.0,
        },
        vel: TDVec {
            x: 0.0,
            y: 0.0,
        },
        dir: PI,
        fire: false,
        thrust_a: 0.0,
        rotate_a: 0.0,
    };

    game.missiles = Vec::new();
    game.rocks = Vec::new();
    game.score = 0;
    game.shipcount = 3;

    let mut rng = rand::thread_rng();
    for _i in 0..10 {
        game.rocks.push(Rock {
            pos: TDVec {
                x: rng.gen_range(0.0..640.0),
                y: rng.gen_range(0.0..480.0),
            },
            vel: TDVec {
                x: rng.gen_range(-1.5..1.5),
                y: rng.gen_range(-1.5..1.5),
            },
            r: 10.0,
        });
    }
}