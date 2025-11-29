use glam::{Mat4, Vec3, Vec4};
use std::f32::consts::PI;

// Astronomical constants
pub const AU_TO_KM: f32 = 149597870.7;
pub const ECLIPTIC_OBLIQUITY: f32 = 23.43928;
pub const J2000_EPOCH: f32 = 2451545.0;
pub const GALACTIC_NORTH_RA: f32 = 192.85948;
pub const GALACTIC_NORTH_DEC: f32 = 27.12825;
pub const GALACTIC_CENTER_L: f32 = 0.0;
pub const GALACTIC_CENTER_B: f32 = 0.0;
pub const SOLAR_APEX_RA: f32 = 277.0;
pub const SOLAR_APEX_DEC: f32 = 30.0;

#[inline]
pub fn deg_to_rad(degrees: f32) -> f32 {
    degrees * PI / 180.0
}

#[inline]
pub fn rad_to_deg(radians: f32) -> f32 {
    radians * 180.0 / PI
}

pub struct CoordinateTransforms;

impl CoordinateTransforms {
    pub fn ecliptic_to_equatorial() -> Mat4 {
        let obliquity = deg_to_rad(ECLIPTIC_OBLIQUITY);
        Mat4::from_rotation_x(obliquity)
    }

    pub fn equatorial_to_ecliptic() -> Mat4 {
        Self::ecliptic_to_equatorial().inverse()
    }

    pub fn galactic_to_equatorial() -> Mat4 {
        // J2000.0
        let l_cp = deg_to_rad(122.93192);
        let ra_gp = deg_to_rad(GALACTIC_NORTH_RA);
        let dec_gp = deg_to_rad(GALACTIC_NORTH_DEC);

        let cos_dec_gp = dec_gp.cos();
        let sin_dec_gp = dec_gp.sin();
        let cos_ra_gp = ra_gp.cos();
        let sin_ra_gp = ra_gp.sin();
        let cos_l_cp = l_cp.cos();
        let sin_l_cp = l_cp.sin();

        // Construct matrix from row vectors (basis vectors of Galactic frame in Equatorial coords)
        // The JS code sets elements explicitly in row-major order.
        // Row 1
        let r1c1 = -sin_l_cp * cos_dec_gp * cos_ra_gp - cos_l_cp * sin_ra_gp;
        let r1c2 = -sin_l_cp * cos_dec_gp * sin_ra_gp + cos_l_cp * cos_ra_gp;
        let r1c3 = sin_l_cp * sin_dec_gp;
        let r1c4 = 0.0;

        // Row 2
        let r2c1 = cos_l_cp * cos_dec_gp * cos_ra_gp - sin_l_cp * sin_ra_gp;
        let r2c2 = cos_l_cp * cos_dec_gp * sin_ra_gp + sin_l_cp * cos_ra_gp;
        let r2c3 = -cos_l_cp * sin_dec_gp;
        let r2c4 = 0.0;

        // Row 3
        let r3c1 = sin_dec_gp * cos_ra_gp;
        let r3c2 = sin_dec_gp * sin_ra_gp;
        let r3c3 = cos_dec_gp;
        let r3c4 = 0.0;

        // Row 4
        let r4c1 = 0.0;
        let r4c2 = 0.0;
        let r4c3 = 0.0;
        let r4c4 = 1.0;

        // glam::Mat4::from_cols_array expects column-major array
        // So we transpose the row-major layout
        Mat4::from_cols_array(&[
            r1c1, r2c1, r3c1, r4c1, // Col 1
            r1c2, r2c2, r3c2, r4c2, // Col 2
            r1c3, r2c3, r3c3, r4c3, // Col 3
            r1c4, r2c4, r3c4, r4c4, // Col 4
        ])
    }

    pub fn equatorial_to_galactic() -> Mat4 {
        Self::galactic_to_equatorial().inverse()
    }

    /// Convert HEE (Heliocentric Earth Ecliptic) to HGI (Heliographic Inertial)
    pub fn hee_to_hgi(position: Vec3, julian_date: f32) -> Vec3 {
        let t = (julian_date - J2000_EPOCH) / 36525.0;

        // Solar rotation parameters
        let theta0 = 100.46 + 36000.77 * t + 0.04107 * t * t; // degrees
        let i = 7.25; // Solar inclination
        let omega = 74.37 + 0.0527 * t; // Longitude of ascending node

        let theta0_rad = deg_to_rad(theta0);
        let i_rad = deg_to_rad(i);
        let omega_rad = deg_to_rad(omega);

        let cos_i = i_rad.cos();
        let sin_i = i_rad.sin();
        let cos_omega = omega_rad.cos();
        let sin_omega = omega_rad.sin();
        let cos_theta0 = theta0_rad.cos();
        let sin_theta0 = theta0_rad.sin();

        // Construct transformation matrix (Row-major in JS)
        // Row 1
        let r1c1 = cos_omega * cos_theta0 - sin_omega * sin_theta0 * cos_i;
        let r1c2 = -cos_omega * sin_theta0 - sin_omega * cos_theta0 * cos_i;
        let r1c3 = sin_omega * sin_i;

        // Row 2
        let r2c1 = sin_omega * cos_theta0 + cos_omega * sin_theta0 * cos_i;
        let r2c2 = -sin_omega * sin_theta0 + cos_omega * cos_theta0 * cos_i;
        let r2c3 = -cos_omega * sin_i;

        // Row 3
        let r3c1 = sin_theta0 * sin_i;
        let r3c2 = cos_theta0 * sin_i;
        let r3c3 = cos_i;

        let transform = Mat4::from_cols_array(&[
            r1c1, r2c1, r3c1, 0.0, r1c2, r2c2, r3c2, 0.0, r1c3, r2c3, r3c3, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]);

        transform.transform_vector3(position)
    }

    /// Convert HGI to HEE
    pub fn hgi_to_hee(position: Vec3, julian_date: f32) -> Vec3 {
        let t = (julian_date - J2000_EPOCH) / 36525.0;

        let theta0 = 100.46 + 36000.77 * t + 0.04107 * t * t;
        let i = 7.25;
        let omega = 74.37 + 0.0527 * t;

        let theta0_rad = deg_to_rad(theta0);
        let i_rad = deg_to_rad(i);
        let omega_rad = deg_to_rad(omega);

        let cos_i = i_rad.cos();
        let sin_i = i_rad.sin();
        let cos_omega = omega_rad.cos();
        let sin_omega = omega_rad.sin();
        let cos_theta0 = theta0_rad.cos();
        let sin_theta0 = theta0_rad.sin();

        // Transpose of HEE to HGI
        // Row 1 (was Col 1 of previous, but wait... previous was constructed manually)
        // Let's trust the JS inverse logic:
        /*
         const transform = new THREE.Matrix4().set(
          cosOmega * cosTheta0 - sinOmega * sinTheta0 * cosI,
          sinOmega * cosTheta0 + cosOmega * sinTheta0 * cosI,
          sinTheta0 * sinI,
          0,

          -cosOmega * sinTheta0 - sinOmega * cosTheta0 * cosI,
          -sinOmega * sinTheta0 + cosOmega * cosTheta0 * cosI,
          cosTheta0 * sinI,
          0,

          sinOmega * sinI,
          -cosOmega * sinI,
          cosI,
          0,

          0, 0, 0, 1
        );
        */

        let r1c1 = cos_omega * cos_theta0 - sin_omega * sin_theta0 * cos_i;
        let r1c2 = sin_omega * cos_theta0 + cos_omega * sin_theta0 * cos_i;
        let r1c3 = sin_theta0 * sin_i;

        let r2c1 = -cos_omega * sin_theta0 - sin_omega * cos_theta0 * cos_i;
        let r2c2 = -sin_omega * sin_theta0 + cos_omega * cos_theta0 * cos_i;
        let r2c3 = cos_theta0 * sin_i;

        let r3c1 = sin_omega * sin_i;
        let r3c2 = -cos_omega * sin_i;
        let r3c3 = cos_i;

        let transform = Mat4::from_cols_array(&[
            r1c1, r2c1, r3c1, 0.0, r1c2, r2c2, r3c2, 0.0, r1c3, r2c3, r3c3, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]);

        transform.transform_vector3(position)
    }

    pub fn ecliptic_to_galactic(position: Vec3) -> Vec3 {
        let equatorial = Self::ecliptic_to_equatorial().transform_vector3(position);
        Self::equatorial_to_galactic().transform_vector3(equatorial)
    }

    pub fn galactic_to_ecliptic(position: Vec3) -> Vec3 {
        let equatorial = Self::galactic_to_equatorial().transform_vector3(position);
        Self::equatorial_to_ecliptic().transform_vector3(equatorial)
    }

    pub fn icrs_to_ecliptic(ra: f32, dec: f32, distance: f32) -> Vec3 {
        let ra_rad = deg_to_rad(ra);
        let dec_rad = deg_to_rad(dec);

        let x = distance * dec_rad.cos() * ra_rad.cos();
        let y = distance * dec_rad.cos() * ra_rad.sin();
        let z = distance * dec_rad.sin();

        let equatorial = Vec3::new(x, y, z);
        Self::equatorial_to_ecliptic().transform_vector3(equatorial)
    }

    pub fn ecliptic_to_icrs(position: Vec3) -> (f32, f32, f32) {
        let equatorial = Self::ecliptic_to_equatorial().transform_vector3(position);

        let distance = equatorial.length();
        let x = equatorial.x;
        let y = equatorial.y;
        let z = equatorial.z;

        let ra = rad_to_deg(y.atan2(x));
        let dec = rad_to_deg((z / distance).asin());

        let ra = if ra < 0.0 { ra + 360.0 } else { ra };

        (ra, dec, distance)
    }

    pub fn create_rtn_basis(position: Vec3, velocity: Vec3) -> Mat4 {
        let r = position.normalize();
        let v = velocity;
        // t = (v - (v . r) * r).normalize()
        let t = (v - r * v.dot(r)).normalize();
        let n = r.cross(t).normalize();

        // Recompute t to ensure orthogonality?
        // JS: const tOrth = new THREE.Vector3().crossVectors(n, r).normalize();
        let t_orth = n.cross(r).normalize();

        // Mat4::from_cols(x, y, z, w) -> Basis vectors
        // RTN typically maps R to X, T to Y, N to Z?
        // JS: new THREE.Matrix4().makeBasis(r, tOrth, n);
        // makeBasis sets columns.

        Mat4::from_cols(
            Vec4::new(r.x, r.y, r.z, 0.0),
            Vec4::new(t_orth.x, t_orth.y, t_orth.z, 0.0),
            Vec4::new(n.x, n.y, n.z, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    pub fn to_rtn(vector: Vec3, position: Vec3, velocity: Vec3) -> Vec3 {
        let rtn_basis = Self::create_rtn_basis(position, velocity);
        let rtn_inverse = rtn_basis.inverse();
        rtn_inverse.transform_vector3(vector)
    }

    pub fn from_rtn(vector_rtn: Vec3, position: Vec3, velocity: Vec3) -> Vec3 {
        let rtn_basis = Self::create_rtn_basis(position, velocity);
        rtn_basis.transform_vector3(vector_rtn)
    }

    pub fn au_to_km(au: f32) -> f32 {
        au * AU_TO_KM
    }

    pub fn km_to_au(km: f32) -> f32 {
        km / AU_TO_KM
    }
}

pub struct AngleUtils;

impl AngleUtils {
    pub fn normalize_degrees(degrees: f32) -> f32 {
        let mut result = degrees % 360.0;
        if result < 0.0 {
            result += 360.0;
        }
        result
    }

    pub fn normalize_radians(radians: f32) -> f32 {
        let mut result = radians % (2.0 * PI);
        if result < 0.0 {
            result += 2.0 * PI;
        }
        result
    }

    pub fn angular_separation(ra1: f32, dec1: f32, ra2: f32, dec2: f32) -> f32 {
        let ra1_rad = deg_to_rad(ra1);
        let dec1_rad = deg_to_rad(dec1);
        let ra2_rad = deg_to_rad(ra2);
        let dec2_rad = deg_to_rad(dec2);

        let d_ra = ra2_rad - ra1_rad;
        let d_dec = dec2_rad - dec1_rad;

        let a = (d_dec / 2.0).sin().powi(2)
            + dec1_rad.cos() * dec2_rad.cos() * (d_ra / 2.0).sin().powi(2);

        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        rad_to_deg(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn test_ecliptic_to_equatorial() {
        let transform = CoordinateTransforms::ecliptic_to_equatorial();
        let obliquity = deg_to_rad(ECLIPTIC_OBLIQUITY);
        let expected_y = Vec3::Y * obliquity.cos() + Vec3::Z * obliquity.sin();

        let transformed_y = transform.transform_vector3(Vec3::Y);

        let diff = (transformed_y - expected_y).length();
        assert!(
            diff < 1e-4,
            "Y axis transformation incorrect. Expected {:?}, got {:?}",
            expected_y,
            transformed_y
        );
    }

    #[test]
    fn test_hee_to_hgi_identity_at_epoch() {
        let pos = Vec3::new(1.0, 0.0, 0.0);
        let jd = J2000_EPOCH;
        let transformed = CoordinateTransforms::hee_to_hgi(pos, jd);
        // We don't know exact value without calculating, but we can check roundtrip
        let back = CoordinateTransforms::hgi_to_hee(transformed, jd);

        let diff = (pos - back).length();
        assert!(
            diff < 1e-4,
            "Roundtrip HEE -> HGI -> HEE failed. Diff: {}",
            diff
        );
    }

    #[test]
    fn test_angular_separation() {
        // Two points on equator, 90 deg apart
        let sep = AngleUtils::angular_separation(0.0, 0.0, 90.0, 0.0);
        assert!((sep - 90.0).abs() < 1e-4);

        // North pole and equator
        let sep2 = AngleUtils::angular_separation(0.0, 90.0, 0.0, 0.0);
        assert!((sep2 - 90.0).abs() < 1e-4);
    }
}
