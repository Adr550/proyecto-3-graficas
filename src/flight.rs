//! Scene state and frame-rate-independent free flight, approach, and local-orbit controls.
use crate::math::{M4, V3};
use crate::renderer::Camera;
use crate::world::{DEFINITIONS, World};
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    Overview,
    Focus,
    Flight,
    Tour,
}
#[derive(Clone, Copy, Default)]
pub struct Input {
    pub forward: f32,
    pub turn: f32,
    pub pitch: f32,
    pub strafe: f32,
    pub lift: f32,
    pub boost: bool,
    pub brake: bool,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub dragging: bool,
    pub steering: bool,
    pub scroll: f32,
}
pub struct Ship {
    pub position: V3,
    pub velocity: V3,
    pub yaw: f32,
    pub pitch: f32,
    target_yaw: f32,
    target_pitch: f32,
    pub local: Option<usize>,
    pub local_yaw: f32,
    pub local_pitch: f32,
    pub local_radius: f32,
    pub gravity: bool,
    pub roll: f32,
}
impl Ship {
    fn new() -> Self {
        Self {
            position: V3::new(55., 40., 170.),
            velocity: V3::ZERO,
            yaw: 0.,
            pitch: 0.,
            target_yaw: 0.,
            target_pitch: 0.,
            local: None,
            local_yaw: 0.,
            local_pitch: 0.35,
            local_radius: 55.,
            gravity: false,
            roll: 0.,
        }
    }
    pub fn forward(&self) -> V3 {
        V3::new(
            -self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        )
    }
    pub fn aim(&mut self, d: V3) {
        let f = d.normalized();
        self.yaw = (-f.x).atan2(-f.z);
        self.pitch = f.y.asin();
        self.target_yaw = self.yaw;
        self.target_pitch = self.pitch;
    }
    pub fn model(&self) -> M4 {
        let f = self.forward();
        let right = V3::new(self.yaw.cos(), 0., -self.yaw.sin());
        let up = right.cross(f).normalized();
        let (sr, cr) = self.roll.sin_cos();
        M4::basis(
            right * cr + up * sr,
            up * cr - right * sr,
            -f,
            self.position,
            0.6,
        )
    }
}
struct Approach {
    start: V3,
    direction: V3,
    target: usize,
    elapsed: f32,
}
pub struct App {
    pub world: World,
    pub mode: Mode,
    pub selected: Option<usize>,
    pub ship: Ship,
    pub camera: Camera,
    pub paused: bool,
    pub time_scale: f32,
    pub paths: bool,
    pub diagnostics: bool,
    pub help: bool,
    pub guide: bool,
    pub visited: [bool; 5],
    pub capture: bool,
    pub quality: usize,
    pub toast: String,
    pub toast_remaining: f32,
    orbit_yaw: f32,
    orbit_pitch: f32,
    orbit_distance: f32,
    focus_distance: f32,
    approach: Option<Approach>,
    pub tour_elapsed: f32,
    tour_stage: usize,
    pub aspect: f32,
}
impl App {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            mode: Mode::Overview,
            selected: None,
            ship: Ship::new(),
            camera: Camera::default(),
            paused: false,
            time_scale: 1.,
            paths: true,
            diagnostics: false,
            help: false,
            guide: false,
            visited: [false; 5],
            capture: false,
            quality: 1,
            toast: String::new(),
            toast_remaining: 0.,
            orbit_yaw: 0.73,
            orbit_pitch: 0.48,
            orbit_distance: 610.,
            focus_distance: 5.5,
            approach: None,
            tour_elapsed: 0.,
            tour_stage: usize::MAX,
            aspect: 1.6,
        }
    }
    pub fn notify(&mut self, message: impl Into<String>) {
        self.toast = message.into();
        self.toast_remaining = 4.;
    }
    /// Navigate from the actual ship position to a moving world; launching is only allowed on arrival.
    pub fn travel_to(&mut self, index: usize) {
        if index >= self.world.planets.len() {
            return;
        }
        let direction = (self.ship.position - self.world.planets[index].pos).normalized();
        self.approach = Some(Approach {
            start: self.ship.position,
            direction,
            target: index,
            elapsed: 0.,
        });
        self.ship.local = None;
        self.selected = Some(index);
        self.mode = Mode::Flight;
        self.capture = false;
        self.ship.velocity = V3::ZERO;
    }
    pub fn face_destination(&mut self) {
        if let Some(i) = self.selected {
            self.approach = None;
            self.ship.local = None;
            self.mode = Mode::Flight;
            self.ship
                .aim(self.world.planets[i].pos - self.ship.position);
        }
    }
    pub fn warp_strength(&self) -> f32 {
        self.approach.as_ref().map_or(0., |a| {
            (a.elapsed / 3.)
                .clamp(0., 1.)
                .mul_add(std::f32::consts::PI, 0.)
                .sin()
                .max(0.)
        })
    }
    pub fn travelling(&self) -> bool {
        self.approach.is_some()
    }
    pub fn docked(&self) -> Option<usize> {
        if self.travelling() || self.mode != Mode::Flight {
            return None;
        }
        let i = self.nearest();
        ((self.ship.position - self.world.planets[i].pos).length()
            <= self.world.planets[i].def.radius * 3.15)
            .then_some(i)
    }
    pub fn overview(&mut self) {
        self.mode = Mode::Overview;
        self.selected = None;
        self.ship.local = None;
        self.approach = None;
        self.capture = false;
    }
    pub fn focus(&mut self, index: usize) {
        if index >= 5 {
            return;
        }
        self.mode = Mode::Focus;
        self.selected = Some(index);
        self.focus_distance = if index == 2 { 6.5 } else { 5.5 };
        self.capture = false;
        self.ship.local = None;
        self.approach = None;
    }
    pub fn start_flight(&mut self, visit: bool) {
        if self.mode == Mode::Flight || self.mode == Mode::Tour {
            self.overview();
            return;
        }
        if let Some(i) = self.selected {
            let p = &self.world.planets[i];
            let direction = (self.camera.eye - p.pos).normalized();
            self.ship.position = p.pos + direction * p.def.radius * 5.8;
            self.ship.velocity = V3::ZERO;
            self.ship.aim(p.pos - self.ship.position);
            if visit {
                self.approach = Some(Approach {
                    start: self.ship.position,
                    direction,
                    target: i,
                    elapsed: 0.,
                });
            }
        } else {
            self.ship.position = V3::new(55., 40., 170.);
            self.ship.velocity = V3::ZERO;
            self.ship.aim(-self.ship.position);
        }
        self.ship.local = None;
        self.mode = Mode::Flight;
        self.capture = !visit;
        self.notify(if visit {
            "Approaching the surface. Local orbit will engage automatically."
        } else {
            "WASD to fly / mouse to steer / Shift to boost / O for local orbit"
        });
    }
    fn engage_local(&mut self, i: usize) {
        let p = &self.world.planets[i];
        let delta = self.ship.position - p.pos;
        let d = delta.length();
        self.ship.local = Some(i);
        self.ship.local_radius = d.max(p.collision_radius() + 10.);
        self.ship.local_yaw = delta.x.atan2(delta.z);
        self.ship.local_pitch = (delta.y / d.max(0.01)).asin();
        self.ship.velocity = V3::ZERO;
        self.capture = false;
        self.notify(format!(
            "Local orbit: {} / A-D to circle / W-S to approach",
            p.def.name
        ));
    }
    pub fn toggle_local(&mut self) {
        if self.mode != Mode::Flight {
            return;
        }
        self.approach = None;
        if self.ship.local.is_some() {
            self.ship.local = None;
            self.capture = true;
            self.notify("Free flight. Mouse to steer, Escape to release.");
            return;
        }
        let i = self.world.nearest(self.ship.position);
        let p = &self.world.planets[i];
        if (self.ship.position - p.pos).length() > p.def.radius + 125. {
            self.notify("Fly closer to a planet to engage local orbit.");
            return;
        }
        self.engage_local(i);
    }
    pub fn start_tour(&mut self) {
        if self.mode == Mode::Tour {
            self.overview();
            return;
        }
        self.mode = Mode::Tour;
        self.selected = None;
        self.tour_elapsed = 0.;
        self.tour_stage = usize::MAX;
        self.ship.local = None;
        self.approach = None;
        self.capture = false;
        self.paused = false;
        self.notify("A sixty-second planet expedition. Every world keeps moving.");
    }
    pub fn update(&mut self, dt: f32, input: Input) {
        self.toast_remaining = (self.toast_remaining - dt).max(0.);
        if self.help || self.guide {
            return;
        }
        // Turn the displayed ship in place while browsing the hub.
        if !self.paused && matches!(self.mode, Mode::Overview | Mode::Focus) {
            self.ship.target_yaw -= input.turn * dt * 1.8;
            self.ship.target_pitch =
                (self.ship.target_pitch + input.pitch * dt * 1.2).clamp(-1.3, 1.3);
            let response = 1. - (-dt * 10.).exp();
            self.ship.yaw += (self.ship.target_yaw - self.ship.yaw) * response;
            self.ship.pitch += (self.ship.target_pitch - self.ship.pitch) * response;
        }
        if input.dragging && (self.mode == Mode::Overview || self.mode == Mode::Focus) {
            self.orbit_yaw -= input.mouse_x * 0.006;
            self.orbit_pitch = (self.orbit_pitch + input.mouse_y * 0.005).clamp(-1.15, 1.25);
        }
        if input.scroll != 0. {
            match self.mode {
                Mode::Overview => {
                    self.orbit_distance =
                        (self.orbit_distance * (-input.scroll * 0.12).exp()).clamp(170., 1300.)
                }
                Mode::Focus => {
                    self.focus_distance =
                        (self.focus_distance * (-input.scroll * 0.1).exp()).clamp(2.5, 13.)
                }
                Mode::Flight => {
                    if let Some(i) = self.ship.local {
                        self.ship.local_radius = (self.ship.local_radius - input.scroll * 3.)
                            .clamp(
                                self.world.planets[i].collision_radius() + 8.,
                                self.world.planets[i].def.radius + 130.,
                            );
                    }
                }
                _ => {}
            }
        }
        if self.mode == Mode::Flight && !self.paused {
            if input.brake
                || input.turn != 0.
                || input.pitch != 0.
                || input.forward != 0.
                || input.strafe != 0.
                || input.lift != 0.
            {
                self.approach = None;
            }
            if self.ship.local.is_none() {
                self.ship.target_yaw -= input.turn * dt * 1.8;
                self.ship.target_pitch =
                    (self.ship.target_pitch + input.pitch * dt * 1.2).clamp(-1.3, 1.3);
            }
            let bank = if self.ship.local.is_some() {
                -(input.turn + input.strafe) * 0.23
            } else {
                -input.turn * 0.3
            };
            self.ship.roll += (bank - self.ship.roll) * (1. - (-dt * 7.).exp());
        }
        if input.steering && self.mode == Mode::Flight && self.ship.local.is_none() {
            self.ship.target_yaw -= input.mouse_x * 0.0026;
            self.ship.target_pitch =
                (self.ship.target_pitch - input.mouse_y * 0.0026).clamp(-1.35, 1.35);
            self.approach = None;
        }
        if !self.paused {
            self.world.update(dt * self.time_scale);
            match self.mode {
                Mode::Flight => {
                    let mut remaining = dt;
                    while remaining > 0. {
                        let h = remaining.min(1. / 120.);
                        self.update_flight(h, input);
                        remaining -= h;
                    }
                }
                Mode::Tour => self.update_tour(dt),
                _ => {}
            }
        }
        let desired = self.desired_camera();
        let response = 1.
            - (-dt
                * if self.mode == Mode::Flight || self.mode == Mode::Tour {
                    7.
                } else {
                    3.6
                })
            .exp();
        self.camera.eye = self.camera.eye.lerp(desired.eye, response);
        self.camera.target = self.camera.target.lerp(desired.target, response);
        self.camera.up = desired.up;
    }
    fn update_flight(&mut self, dt: f32, input: Input) {
        if let Some(approach) = self.approach.as_mut() {
            approach.elapsed += dt;
            let t = (approach.elapsed / 3.).clamp(0., 1.);
            let ease = t * t * (3. - 2. * t);
            let p = &self.world.planets[approach.target];
            let destination = p.pos + approach.direction * p.def.radius * 2.8;
            self.ship.position = approach.start.lerp(destination, ease)
                + V3::Y * (t * std::f32::consts::PI).sin() * 55.;
            self.ship.velocity = V3::ZERO;
            self.ship.aim(p.pos - self.ship.position);
            if t >= 1. {
                let i = approach.target;
                self.approach = None;
                self.engage_local(i);
                self.visited[i] = true;
            }
            return;
        }
        if let Some(i) = self.ship.local {
            let p = &self.world.planets[i];
            self.ship.local_yaw += (input.strafe + input.turn) * dt * 0.8;
            self.ship.local_pitch =
                (self.ship.local_pitch + (input.lift + input.pitch) * dt * 0.4).clamp(-1.25, 1.25);
            self.ship.local_radius = (self.ship.local_radius - input.forward * dt * 17.)
                .clamp(p.collision_radius() + 8., p.def.radius + 130.);
            let a = self.ship.local_yaw;
            let t = self.ship.local_pitch;
            let d = V3::new(a.sin() * t.cos(), t.sin(), a.cos() * t.cos());
            let old = self.ship.position;
            self.ship.position = p.pos + d * self.ship.local_radius;
            self.ship.velocity = (self.ship.position - old) / dt.max(0.001);
            self.ship.aim(-d);
        } else {
            let response = 1. - (-dt * 10.).exp();
            self.ship.yaw += (self.ship.target_yaw - self.ship.yaw) * response;
            self.ship.pitch += (self.ship.target_pitch - self.ship.pitch) * response;
            let f = self.ship.forward();
            let right = V3::new(self.ship.yaw.cos(), 0., -self.ship.yaw.sin());
            let up = right.cross(f).normalized();
            let mut movement = f * input.forward + right * input.strafe + up * input.lift;
            if movement.length() > 1. {
                movement = movement.normalized();
            }
            let desired = movement * if input.boost { 115. } else { 44. };
            self.ship.velocity = self.ship.velocity.lerp(desired, 1. - (-dt * 4.).exp());
            if self.ship.gravity && !input.brake {
                for p in &self.world.planets {
                    let delta = p.pos - self.ship.position;
                    let distance = delta.length();
                    if distance < p.def.radius * 6. && distance > 1. {
                        self.ship.velocity += delta
                            * (p.def.radius * p.def.radius * 2.2 / (distance * distance) * dt
                                / distance);
                    }
                }
            }
            if input.brake {
                self.ship.velocity = self.ship.velocity * (-dt * 7.).exp();
            }
            self.ship.position += self.ship.velocity * dt;
        }
        let (position, velocity, _) = self.world.collide(self.ship.position, self.ship.velocity);
        self.ship.position = position;
        self.ship.velocity = velocity;
        if self.ship.position.length() > 1100. {
            self.ship.position = self.ship.position.normalized() * 1095.;
            self.ship.velocity = V3::ZERO;
            self.notify("The edge of the atlas. Turn back toward the sun.");
        }
        let i = self
            .ship
            .local
            .unwrap_or_else(|| self.world.nearest(self.ship.position));
        if (self.ship.position - self.world.planets[i].pos).length()
            < self.world.planets[i].def.radius * 3.7
        {
            self.visited[i] = true;
        }
    }
    fn update_tour(&mut self, dt: f32) {
        self.tour_elapsed += dt;
        if self.tour_elapsed >= 60. {
            self.overview();
            self.notify("Five worlds. One continuous journey. Expedition complete.");
            return;
        }
        let stage = (self.tour_elapsed / 12.).floor() as usize;
        let i = [1, 2, 0, 3, 4][stage];
        let u = (self.tour_elapsed % 12.) / 12.;
        if stage != self.tour_stage {
            self.tour_stage = stage;
            self.notify(format!(
                "{} / {}",
                DEFINITIONS[i].name, DEFINITIONS[i].subtitle
            ));
        }
        self.demo_pose(i, u);
        self.visited[i] = true;
    }
    pub fn demo_pose(&mut self, index: usize, u: f32) {
        let p = &self.world.planets[index];
        let ratio = if u < 0.25 {
            6.5 - u / 0.25 * 3.5
        } else if u > 0.8 {
            3. + (u - 0.8) / 0.2 * 3.5
        } else {
            3.
        };
        let a = 0.6 + u * std::f32::consts::TAU * 1.1;
        let distance = p.def.radius * ratio;
        let offset = V3::new(
            a.sin() * distance,
            (0.35 + (u * std::f32::consts::TAU).sin() * 0.15) * distance,
            a.cos() * distance,
        );
        self.ship.position = p.pos + offset;
        self.ship.aim(-offset);
        self.ship.velocity = V3::new(-a.cos(), 0., a.sin()) * 15.;
    }
    pub fn desired_camera(&self) -> Camera {
        match self.mode {
            Mode::Overview => {
                let d = self.orbit_distance * (1.15 / self.aspect).max(1.);
                Camera {
                    eye: V3::new(
                        self.orbit_yaw.sin() * self.orbit_pitch.cos() * d,
                        self.orbit_pitch.sin() * d,
                        self.orbit_yaw.cos() * self.orbit_pitch.cos() * d,
                    ),
                    ..Camera::default()
                }
            }
            Mode::Focus => {
                let p = &self.world.planets[self.selected.unwrap_or(0)];
                let offset = V3::new(
                    self.orbit_yaw.sin() * self.orbit_pitch.cos(),
                    self.orbit_pitch.sin(),
                    self.orbit_yaw.cos() * self.orbit_pitch.cos(),
                );
                let right = V3::new(offset.z, 0., -offset.x).normalized();
                let target =
                    p.pos - right * p.def.radius * if self.aspect < 1.3 { 0.65 } else { 1.05 };
                Camera {
                    eye: target + offset * (p.def.radius * self.focus_distance),
                    target,
                    ..Camera::default()
                }
            }
            Mode::Flight | Mode::Tour => {
                let f = self.ship.forward();
                let right = V3::new(self.ship.yaw.cos(), 0., -self.ship.yaw.sin());
                let up = right.cross(f).normalized();
                Camera {
                    eye: self.ship.position - f * 24. + up * 6.,
                    target: self.ship.position + f * 40.,
                    up,
                    fov: 0.8,
                }
            }
        }
    }
    pub fn ship_model(&self) -> Option<M4> {
        if self.mode == Mode::Flight || self.mode == Mode::Tour {
            Some(self.ship.model())
        } else {
            None
        }
    }
    pub fn nearest(&self) -> usize {
        self.ship
            .local
            .unwrap_or_else(|| self.world.nearest(self.ship.position))
    }
}
impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hub_arrows_rotate_ship_in_place_and_respect_pause() {
        for mode in [Mode::Overview, Mode::Focus] {
            let mut app = App::new();
            app.mode = mode;
            app.help = false;
            app.guide = false;
            let position = app.ship.position;
            let yaw = app.ship.yaw;
            let pitch = app.ship.pitch;
            for _ in 0..60 {
                app.update(
                    1. / 60.,
                    Input {
                        turn: 1.,
                        pitch: 1.,
                        ..Input::default()
                    },
                );
            }
            assert!(app.ship.yaw < yaw - 1.);
            assert!(app.ship.pitch > pitch + 0.5);
            assert_eq!(app.ship.position, position);
            app.paused = true;
            let yaw = app.ship.yaw;
            app.update(
                1.,
                Input {
                    turn: -1.,
                    ..Input::default()
                },
            );
            assert_eq!(app.ship.yaw, yaw);
        }
    }
    #[test]
    fn seamless_local_orbit_follows_moving_planet() {
        let mut app = App::new();
        app.focus(1);
        for _ in 0..120 {
            app.update(1. / 60., Input::default());
        }
        app.start_flight(true);
        for _ in 0..300 {
            app.update(1. / 60., Input::default());
        }
        assert_eq!(app.ship.local, Some(1));
        let r = (app.ship.position - app.world.planets[1].pos).length();
        for _ in 0..180 {
            app.update(1. / 60., Input::default());
        }
        assert!(((app.ship.position - app.world.planets[1].pos).length() - r).abs() < 0.1);
    }
    #[test]
    fn keyboard_turning_changes_ship_heading_and_bank() {
        let mut app = App::new();
        app.mode = Mode::Flight;
        let old = app.ship.forward();
        for _ in 0..60 {
            app.update(
                1. / 60.,
                Input {
                    turn: 1.,
                    ..Input::default()
                },
            );
        }
        assert!((app.ship.forward() - old).length() > 1.);
        assert!(app.ship.roll < -0.2);
    }
    #[test]
    fn manual_turn_cancels_autopilot_without_teleporting() {
        let mut app = App::new();
        app.travel_to(2);
        app.update(0.1, Input::default());
        let old = app.ship.position;
        app.update(
            1. / 60.,
            Input {
                turn: 1.,
                ..Input::default()
            },
        );
        assert!(!app.travelling());
        assert!((app.ship.position - old).length() < 1.);
    }
    #[test]
    fn pause_and_tour() {
        let mut app = App::new();
        app.paused = true;
        app.update(1., Input::default());
        assert_eq!(app.world.elapsed, 0.);
        app.start_tour();
        for _ in 0..3610 {
            app.update(1. / 60., Input::default());
        }
        assert_eq!(app.mode, Mode::Overview);
        assert!(app.visited.iter().all(|v| *v));
    }
}
