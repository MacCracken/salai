//! Entity picking via raycasting — click in viewport to select entities.
//!
//! Casts a ray from the camera through the click point and tests against
//! entity positions (sphere intersection) using [`hisab::geo`].

use hisab::{Mat4, Vec3};
use kiran::scene::Position;

/// Result of a pick operation.
#[derive(Debug, Clone)]
pub struct PickResult {
    /// The entity that was hit.
    pub entity: kiran::Entity,
    /// Distance from the camera to the hit point.
    pub distance: f32,
    /// World-space position of the entity.
    pub position: Vec3,
}

/// Cast a ray from camera through a viewport point and find the nearest entity hit.
///
/// `ndc_x` and `ndc_y` are in normalized device coordinates (-1 to 1).
/// `pick_radius` is the radius of the sphere around each entity position for hit testing.
#[must_use]
pub fn pick_entity(
    world: &kiran::World,
    entities: &[kiran::Entity],
    camera_pos: Vec3,
    view_proj: Mat4,
    ndc_x: f32,
    ndc_y: f32,
    pick_radius: f32,
) -> Option<PickResult> {
    let ray = screen_to_ray(camera_pos, view_proj, ndc_x, ndc_y)?;

    let mut closest: Option<PickResult> = None;

    for &entity in entities {
        if !world.is_alive(entity) {
            continue;
        }
        let Some(pos) = world.get_component::<Position>(entity) else {
            continue;
        };

        if let Some(t) = ray_sphere_intersect(ray.0, ray.1, pos.0, pick_radius)
            && closest.as_ref().is_none_or(|c| t < c.distance)
        {
            closest = Some(PickResult {
                entity,
                distance: t,
                position: pos.0,
            });
        }
    }

    closest
}

/// Convert viewport click to a world-space ray.
///
/// Returns `(origin, direction)` or `None` if the inverse matrix is degenerate.
#[must_use]
#[inline]
fn screen_to_ray(
    camera_pos: Vec3,
    view_proj: Mat4,
    ndc_x: f32,
    ndc_y: f32,
) -> Option<(Vec3, Vec3)> {
    let det = view_proj.determinant();
    if det.abs() < f32::EPSILON {
        return None;
    }
    let inv = view_proj.inverse();

    let near = inv.project_point3(Vec3::new(ndc_x, ndc_y, -1.0));
    let far = inv.project_point3(Vec3::new(ndc_x, ndc_y, 1.0));

    let direction = (far - near).normalize();
    if direction.x.is_nan() {
        return None;
    }

    Some((camera_pos, direction))
}

/// Ray-sphere intersection. Returns distance along ray or None.
#[must_use]
#[inline]
fn ray_sphere_intersect(origin: Vec3, direction: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let oc = origin - center;
    let a = direction.dot(direction);
    let b = 2.0 * oc.dot(direction);
    let c = oc.dot(oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        return None;
    }

    let sqrt_d = discriminant.sqrt();
    let t1 = (-b - sqrt_d) / (2.0 * a);
    let t2 = (-b + sqrt_d) / (2.0 * a);

    if t1 >= 0.0 {
        Some(t1)
    } else if t2 >= 0.0 {
        Some(t2)
    } else {
        None // sphere is behind the ray
    }
}

/// Convert viewport pixel coordinates to NDC.
///
/// `x`, `y` are pixel coordinates. `width`, `height` are viewport dimensions.
#[must_use]
#[inline]
pub fn pixel_to_ndc(x: f32, y: f32, width: f32, height: f32) -> (f32, f32) {
    if width < f32::EPSILON || height < f32::EPSILON {
        return (0.0, 0.0);
    }
    let ndc_x = (2.0 * x / width) - 1.0;
    let ndc_y = 1.0 - (2.0 * y / height); // Y is flipped
    (ndc_x, ndc_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_sphere_hit() {
        let origin = Vec3::new(0.0, 0.0, 5.0);
        let direction = Vec3::new(0.0, 0.0, -1.0);
        let center = Vec3::ZERO;
        let radius = 1.0;

        let t = ray_sphere_intersect(origin, direction, center, radius).unwrap();
        assert!((t - 4.0).abs() < 0.01); // hits at z=1
    }

    #[test]
    fn ray_sphere_miss() {
        let origin = Vec3::new(0.0, 0.0, 5.0);
        let direction = Vec3::new(0.0, 0.0, -1.0);
        let center = Vec3::new(10.0, 0.0, 0.0); // far off to the side
        let radius = 1.0;

        assert!(ray_sphere_intersect(origin, direction, center, radius).is_none());
    }

    #[test]
    fn ray_sphere_behind() {
        let origin = Vec3::new(0.0, 0.0, 5.0);
        let direction = Vec3::new(0.0, 0.0, 1.0); // pointing away
        let center = Vec3::ZERO;
        let radius = 1.0;

        assert!(ray_sphere_intersect(origin, direction, center, radius).is_none());
    }

    #[test]
    fn ray_sphere_inside() {
        let origin = Vec3::ZERO; // inside the sphere
        let direction = Vec3::new(0.0, 0.0, -1.0);
        let center = Vec3::ZERO;
        let radius = 5.0;

        let t = ray_sphere_intersect(origin, direction, center, radius).unwrap();
        assert!(t > 0.0);
    }

    #[test]
    fn pixel_to_ndc_center() {
        let (x, y) = pixel_to_ndc(640.0, 360.0, 1280.0, 720.0);
        assert!(x.abs() < 0.01);
        assert!(y.abs() < 0.01);
    }

    #[test]
    fn pixel_to_ndc_corners() {
        let (x, y) = pixel_to_ndc(0.0, 0.0, 1280.0, 720.0);
        assert!((x - (-1.0)).abs() < 0.01);
        assert!((y - 1.0).abs() < 0.01);

        let (x, y) = pixel_to_ndc(1280.0, 720.0, 1280.0, 720.0);
        assert!((x - 1.0).abs() < 0.01);
        assert!((y - (-1.0)).abs() < 0.01);
    }

    #[test]
    fn pick_entity_finds_nearest() {
        let mut world = kiran::World::new();
        let near = world.spawn();
        let far = world.spawn();
        world
            .insert_component(near, Position(Vec3::new(0.0, 0.0, -3.0)))
            .unwrap();
        world
            .insert_component(far, Position(Vec3::new(0.0, 0.0, -10.0)))
            .unwrap();

        let camera_pos = Vec3::new(0.0, 0.0, 0.0);
        let view_proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 16.0 / 9.0, 0.1, 100.0)
            * Mat4::look_at_rh(camera_pos, Vec3::new(0.0, 0.0, -1.0), Vec3::Y);

        let result = pick_entity(&world, &[near, far], camera_pos, view_proj, 0.0, 0.0, 1.0);
        let hit = result.unwrap();
        assert_eq!(hit.entity, near); // nearer entity selected
    }

    #[test]
    fn pick_entity_no_hit() {
        let mut world = kiran::World::new();
        let e = world.spawn();
        world
            .insert_component(e, Position(Vec3::new(100.0, 0.0, 0.0)))
            .unwrap();

        let camera_pos = Vec3::new(0.0, 0.0, 5.0);
        let view_proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 16.0 / 9.0, 0.1, 100.0)
            * Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y);

        let result = pick_entity(&world, &[e], camera_pos, view_proj, 0.0, 0.0, 0.5);
        assert!(result.is_none());
    }

    #[test]
    fn pick_entity_empty_world() {
        let world = kiran::World::new();
        let result = pick_entity(&world, &[], Vec3::ZERO, Mat4::IDENTITY, 0.0, 0.0, 1.0);
        assert!(result.is_none());
    }

    #[test]
    fn pick_entity_skips_dead() {
        let mut world = kiran::World::new();
        let e = world.spawn();
        world.insert_component(e, Position(Vec3::ZERO)).unwrap();
        world.despawn(e).unwrap();

        let result = pick_entity(
            &world,
            &[e],
            Vec3::new(0.0, 0.0, 5.0),
            Mat4::IDENTITY,
            0.0,
            0.0,
            1.0,
        );
        assert!(result.is_none());
    }

    #[test]
    fn pixel_to_ndc_zero_dimensions() {
        let (x, y) = pixel_to_ndc(100.0, 100.0, 0.0, 0.0);
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn pick_entity_degenerate_matrix() {
        let mut world = kiran::World::new();
        let e = world.spawn();
        world.insert_component(e, Position(Vec3::ZERO)).unwrap();

        let result = pick_entity(
            &world,
            &[e],
            Vec3::new(0.0, 0.0, 5.0),
            Mat4::ZERO,
            0.0,
            0.0,
            1.0,
        );
        assert!(result.is_none());
    }
}
