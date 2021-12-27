use crate::sdk;
use crate::sdk::interfaces::{self, INTERFACES};

fn calculate_angle(src: &cgmath::Vector3<f32>, dst: &cgmath::Vector3<f32>) -> cgmath::Vector3<f32> {
    let delta = cgmath::vec3(src.x - dst.x, src.y - dst.y, src.z - dst.z);
    let hyp = (delta.x * delta.x + delta.y * delta.y).sqrt();

    let mut vangle = cgmath::Vector3::new(
        (delta.z / hyp).atan() * 57.295779513082,
        (delta.y / delta.x).atan() * 57.295779513082,
        0.0,
    );

    if delta.x >= 0.0 {
        vangle.y += 180.0;
    }

    vangle
}

pub fn normalize_angles(angle: &mut cgmath::Vector3<f32>) {
    while angle.x > 89.0 {
        angle.x -= 180.0;
    }

    while angle.x < -89.0 {
        angle.x += 180.0;
    }

    while angle.y > 180.0 {
        angle.y -= 360.0;
    }

    while angle.y < -180.0 {
        angle.y += 360.0;
    }
}

fn angle_length(angle: &cgmath::Vector3<f32>) -> f32 {
    (angle.x * angle.x + angle.y * angle.y + angle.x * angle.z).sqrt()
}

/// Checks whether a player is visible, given anything bones.
pub fn visibility_check(local_player: crate::sdk::entity::CEntity, player: crate::sdk::entity::CEntity, bone_matrix: &[vecmath::Matrix3x4<f32>; 128]) -> bool {
    let mut filter: interfaces::enginetrace::CTraceFilter = interfaces::enginetrace::CTraceFilter::new(local_player.base);

    let head =  cgmath::vec3(
        bone_matrix[8][0][3],
        bone_matrix[8][1][3],
        bone_matrix[8][2][3],
    );

    let mut trace_to_head: interfaces::enginetrace::Trace = unsafe { std::mem::zeroed() };
    let mut ray_to_head = interfaces::enginetrace::Ray::new(local_player.calculate_eye_position(), head);

    INTERFACES.trace.trace_ray(&mut ray_to_head, 0x1 | 0x80 | 0x4000 | 0x2000, &mut filter, &mut trace_to_head);

    let spine =  cgmath::vec3(
        bone_matrix[6][0][3],
        bone_matrix[6][1][3],
        bone_matrix[6][2][3],
    );

    let mut trace_to_spine: interfaces::enginetrace::Trace = unsafe { std::mem::zeroed() };
    let mut ray_to_spine = interfaces::enginetrace::Ray::new(local_player.calculate_eye_position(), spine);

    INTERFACES.trace.trace_ray(&mut ray_to_spine, 0x1 | 0x80 | 0x4000 | 0x2000, &mut filter, &mut trace_to_spine);

    (trace_to_head.m_pEntityHit == player.base || trace_to_spine.m_pEntityHit == player.base)
}

static mut SHOT_LAST_TICK: bool = false;

pub fn create_move(cmd: *mut crate::sdk::interfaces::clientmode::CUserCmd) {
    if INTERFACES.engine.is_in_game() {
        let local_player = INTERFACES
            .entitylist
            .get_client_entity(INTERFACES.engine.get_local_player());
        if local_player.is_null() {
            return;
        }

        let local_player = unsafe { sdk::entity::CEntity::from_raw(local_player) };

        if local_player.get_health() > 0 {
            let weapon = interfaces::entitylist::get_entity_by_id(
                (local_player.active_weapon() as usize & 0xFFF) as i32,
            );
            if weapon.is_null() {
                return;
            }
            const SMOOTHING: f32 = 1.0; //1.0 + (32.0 / 5.0);
            const FOV: f32 = 1000.0; //30.0 / 10.0;

            let mut closest_delta = std::f32::MAX;
            let mut angle_to_closest_bone = cgmath::Vector3::new(0.0, 0.0, 0.0);

            let max_clients = interfaces::INTERFACES.engine.get_max_clients();
            for i in 1..max_clients {
                let entity = unsafe {
                    sdk::entity::CEntity::from_raw(
                        interfaces::INTERFACES.entitylist.get_client_entity(i),
                    )
                };

                if !entity.is_empty() && entity.base != local_player.base {
                    if entity.get_health() > 0
                        && !entity.is_dormant()
                        && local_player.get_team_num() != entity.get_team_num()
                    {
                        let mut bone_matrix: [vecmath::Matrix3x4<f32>; 128] =
                            unsafe { std::mem::zeroed() }; // SAFETY: all values will be initialized
                        if entity.setup_bones(&mut bone_matrix as _, 128, 0x000FFF00, 0.0) {
                            if !visibility_check(local_player, entity, &bone_matrix) {
                                continue;
                            }
                            let local_player_eye_position = local_player.calculate_eye_position();

                            let model = unsafe {
                                &mut *INTERFACES.modelinfo.get_studio_model(&*entity.get_model())
                            };

                            if !(model as *mut interfaces::modelinfo::StudioHdr).is_null() {
                                let hitbox = 8;
                                let target_bone_position = cgmath::vec3(
                                    bone_matrix[hitbox][0][3],
                                    bone_matrix[hitbox][1][3],
                                    bone_matrix[hitbox][2][3],
                                );

                                let mut angle_to_current_bone = calculate_angle(
                                    &local_player_eye_position,
                                    &target_bone_position,
                                ) - unsafe { (*cmd).viewangles }
                                    - ((local_player.get_aim_punch() * 2.0));
                                normalize_angles(&mut angle_to_current_bone);

                                if angle_length(&angle_to_current_bone) < closest_delta {
                                    closest_delta = angle_length(&angle_to_current_bone);
                                    angle_to_closest_bone = angle_to_current_bone;
                                }
                            }
                        }
                    }
                }
            }

            if closest_delta < FOV {
                unsafe {
                    //(*cmd).viewangles += (angle_to_closest_bone / (SMOOTHING / (closest_delta / 2.0)));
                    if !SHOT_LAST_TICK {
                        (*cmd).buttons |= 1 << 0;
                        SHOT_LAST_TICK = true;
                    } else {
                        SHOT_LAST_TICK = false;
                    }
                    (*cmd).viewangles += angle_to_closest_bone / SMOOTHING;
                }
            } else {
                unsafe {
                    SHOT_LAST_TICK = false;
                }
            }
        }
    }
}
