use rand::Rng;

use crate::constants::{MAP_HEIGHT, MAP_WIDTH};
use crate::map_geometry::outbound_route_len;

use super::{DEFAULT_MISSION_SILVER_POOL, GatherMission};

/// Missions with `silver_remaining == 0` are ignored. Tie-break: shortest `outbound_route_len`, then lexicographic `(row, col)` on `cell`, then lower index (same policy as the old tuple picker).
pub fn pick_closest_gather_mission_index(
    from: (u16, u16),
    missions: &[GatherMission],
) -> Option<usize> {
    let mut best_i: Option<usize> = None;
    let mut best_len = usize::MAX;
    let mut best_key = (u16::MAX, u16::MAX);
    for (i, m) in missions.iter().enumerate() {
        if m.silver_remaining == 0 {
            continue;
        }
        let len = outbound_route_len(from, m.cell);
        let key = (m.cell.1, m.cell.0);
        let better = match best_i {
            None => true,
            Some(j) => {
                len < best_len
                    || (len == best_len && key < best_key)
                    || (len == best_len && key == best_key && i < j)
            }
        };
        if better {
            best_len = len;
            best_i = Some(i);
            best_key = key;
        }
    }
    best_i
}

pub fn generate_base_and_three_missions<R: Rng>(rng: &mut R) -> ((u16, u16), Vec<GatherMission>) {
    loop {
        let bc = rng.gen_range(0..MAP_WIDTH);
        let br = rng.gen_range(0..MAP_HEIGHT);
        let base = (bc, br);
        let mut missions: Vec<GatherMission> = Vec::with_capacity(3);
        let mut ok = true;
        for _ in 0..3 {
            let mut tries = 0u32;
            loop {
                let c = rng.gen_range(0..MAP_WIDTH);
                let r = rng.gen_range(0..MAP_HEIGHT);
                let cell = (c, r);
                tries += 1;
                if tries > 10_000 {
                    ok = false;
                    break;
                }
                if cell != base && !missions.iter().any(|m| m.cell == cell) {
                    missions.push(GatherMission::new(cell, DEFAULT_MISSION_SILVER_POOL));
                    break;
                }
            }
            if !ok {
                break;
            }
        }
        if ok && missions.len() == 3 {
            return (base, missions);
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use crate::constants::{MAP_HEIGHT, MAP_WIDTH};
    use crate::map_geometry::outbound_route_len;

    use super::*;

    #[test]
    fn pick_closest_tie_breaks_by_row_then_col() {
        let base = (50u16, 50u16);
        let mut pair: Option<((u16, u16), (u16, u16))> = None;
        'outer: for r1 in 0u16..80u16 {
            for c1 in 0u16..80u16 {
                let a = (c1, r1);
                if a == base {
                    continue;
                }
                let la = outbound_route_len(base, a);
                for r2 in 0u16..80u16 {
                    for c2 in 0u16..80u16 {
                        let b = (c2, r2);
                        if b == base || b == a {
                            continue;
                        }
                        if outbound_route_len(base, b) == la {
                            pair = Some((a, b));
                            break 'outer;
                        }
                    }
                }
            }
        }
        let (a, b) = pair.expect("find two distinct cells with equal route len from base");
        assert_eq!(outbound_route_len(base, a), outbound_route_len(base, b));
        let missions = vec![
            GatherMission::new(a, DEFAULT_MISSION_SILVER_POOL),
            GatherMission::new(b, DEFAULT_MISSION_SILVER_POOL),
        ];
        let idx = pick_closest_gather_mission_index(base, &missions).unwrap();
        let want = if (a.1, a.0) <= (b.1, b.0) { 0 } else { 1 };
        assert_eq!(idx, want);
    }

    #[test]
    fn generates_three_distinct_missions_and_base() {
        let mut rng = StdRng::seed_from_u64(12345);
        let (base, missions) = generate_base_and_three_missions(&mut rng);

        assert_eq!(missions.len(), 3);
        assert!(base.0 < MAP_WIDTH);
        assert!(base.1 < MAP_HEIGHT);
        for (i, mission) in missions.iter().enumerate() {
            let cell = mission.cell;
            assert_ne!(cell, base);
            assert!(cell.0 < MAP_WIDTH);
            assert!(cell.1 < MAP_HEIGHT);
            assert!(!missions[..i].iter().any(|m| m.cell == cell));
            assert_eq!(mission.silver_initial, DEFAULT_MISSION_SILVER_POOL);
            assert_eq!(mission.silver_remaining, DEFAULT_MISSION_SILVER_POOL);
        }
    }
}
