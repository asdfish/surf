use bitvec::array::BitArray;
use fyrox::{
    core::{reflect::prelude::*, type_traits::prelude::*, visitor::prelude::*},
    event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent},
    graph::SceneGraph,
    keyboard::{KeyCode, PhysicalKey},
    renderer::framework::core::{
        algebra::{UnitQuaternion, UnitVector3, Vector3},
        pool::Handle,
    },
    scene::{node::Node, rigidbody::RigidBody},
    script::{ScriptContext, ScriptDeinitContext, ScriptTrait},
};
use std::{
    any::{Any, type_name},
    ops::{Add, Sub},
};

#[derive(Clone, Copy, Debug, Default)]
struct KeyState(BitArray<[u8; 1]>);
impl KeyState {
    pub fn get(&self, idx: Key) -> bool {
        self.0[idx as usize]
    }
    pub fn set(&mut self, idx: Key, with: bool) {
        self.0.set(idx as usize, with);
    }
}
#[derive(Clone, Copy)]
#[repr(u8)]
enum Key {
    W,
    A,
    S,
    D,
}
impl Key {
    pub const VARIANTS: [Self; 4] = [Self::W, Self::S, Self::A, Self::D];

    pub const fn try_from_key_code(key_code: KeyCode) -> Option<Self> {
        match key_code {
            KeyCode::KeyW => Some(Self::W),
            KeyCode::KeyA => Some(Self::A),
            KeyCode::KeyS => Some(Self::S),
            KeyCode::KeyD => Some(Self::D),
            _ => None,
        }
    }
}
impl From<Key> for (MovementDirection, Movement) {
    fn from(key: Key) -> (MovementDirection, Movement) {
        match key {
            Key::W => (MovementDirection::Z, Movement::Positive),
            Key::S => (MovementDirection::Z, Movement::Negative),
            Key::A => (MovementDirection::X, Movement::Positive),
            Key::D => (MovementDirection::X, Movement::Negative),
        }
    }
}
#[derive(Clone, Copy)]
#[repr(u8)]
enum MovementDirection {
    Z,
    X,
}
impl MovementDirection {
    pub const fn get_vector(&self, vectors: [Vector3<f32>; 2]) -> Vector3<f32> {
        vectors[*self as usize]
    }
}
#[derive(Clone, Copy)]
#[repr(u8)]
enum Movement {
    Positive,
    Negative,
}
impl Movement {
    pub const fn operation<T>(&self) -> fn(T, T) -> T
    where
        T: Add<Output = T> + Sub<Output = T>,
    {
        [T::add, T::sub][*self as usize]
    }
}

#[derive(Visit, Reflect, Default, Debug, Clone, TypeUuidProvider, ComponentProvider)]
#[type_uuid(id = "56c83c6f-ec84-4bca-90ac-b5e7ab602b34")]
#[visit(optional)]
pub struct Player {
    // Add fields here.
    camera: Handle<Node>,

    yaw: f32,
    pitch: f32,

    #[reflect(hidden)]
    #[visit(skip)]
    key_state: KeyState,
}

impl ScriptTrait for Player {
    fn on_init(&mut self, context: &mut ScriptContext) {
        // Put initialization logic here.
    }

    fn on_start(&mut self, context: &mut ScriptContext) {
        // There should be a logic that depends on other scripts in scene.
        // It is called right after **all** scripts were initialized.
    }

    fn on_deinit(&mut self, context: &mut ScriptDeinitContext) {
        // Put de-initialization logic here.
    }

    fn on_os_event(&mut self, event: &Event<()>, context: &mut ScriptContext) {
        // Respond to OS events here.
        match event {
            Event::DeviceEvent {
                event:
                    DeviceEvent::MouseMotion {
                        delta: (dx, dy), ..
                    },
                ..
            } => {
                const MOUSE_SPEED: f32 = 0.35;
                self.pitch = (self.pitch + *dy as f32 * MOUSE_SPEED).clamp(-89.9, 89.9);
                self.yaw -= *dx as f32 * MOUSE_SPEED;
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(key_code),
                                state,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                let _ = Key::try_from_key_code(*key_code)
                    .map(move |key_code| (key_code, ElementState::Pressed.eq(state)))
                    .map(|(key_code, state)| self.key_state.set(key_code, state));
            }
            _ => {}
        }
    }

    fn on_update(&mut self, context: &mut ScriptContext) {
        let (look_vector, side_vector) = context
            .scene
            .graph
            .try_get_mut(self.camera)
            .map(|camera| {
                let yaw =
                    UnitQuaternion::from_axis_angle(&Vector3::y_axis(), self.yaw.to_radians());
                camera.local_transform_mut().set_rotation(
                    UnitQuaternion::from_axis_angle(
                        &UnitVector3::new_normalize(yaw * Vector3::x()),
                        self.pitch.to_radians(),
                    ) * yaw,
                );

                (camera.look_vector(), camera.side_vector())
            })
            .unwrap_or_default();

        let velocity = Key::VARIANTS
            .into_iter()
            .filter(|key| self.key_state.get(*key))
            .map(<(MovementDirection, Movement)>::from)
            .map(move |(direction, movement)| {
                (
                    direction.get_vector([look_vector, side_vector]),
                    movement.operation::<Vector3<f32>>(),
                )
            })
            .fold(Vector3::new(0., 0., 0.), |vector, (with, operation)| {
                operation(vector, with)
            });

        context
            .scene
            .graph
            .try_get_mut_of_type::<RigidBody>(context.handle)
            .map(|body| {
                let y_vel = body.lin_vel().y;

                if let Some(normalized_velocity) = velocity.try_normalize(f32::EPSILON) {
                    let movement_speed = 240. * context.dt;
                    body.set_lin_vel(Vector3::new(
                        normalized_velocity.x * movement_speed,
                        y_vel,
                        normalized_velocity.z * movement_speed,
                    ))
                } else {
                    body.set_lin_vel(Vector3::new(0., y_vel, 0.))
                }
            });

        // Put object logic here.
    }
}
