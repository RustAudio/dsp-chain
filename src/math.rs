
use std::mem;
use std::rand;
use std::num;
use std::num::One;


/// Map a value from range in to range out.
pub fn map(val: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    if (in_min - in_max).abs() < Float::epsilon() {
        println!("jmath Warning: map(): avoiding possible divide by zero, in_min ({}) and in_max({})", in_min, in_max);
        return out_min;
    }
    (val - in_min) / (in_max - in_min) * (out_max - out_min) + out_min
}

/// Clamp a value to a range.
pub fn clamp(val: f32, min: f32, max: f32) -> f32 {
    if val < min { min } else { if val > max { max } else { val } }
}

/// Check if value is in range.
pub fn inrange(val: f32, min: f32, max: f32) -> bool {
    val >= min && val <= max
}

/// Interpolate from start to stop 'amt' amount.
pub fn lerp(start: f32, stop: f32, amt: f32) -> f32 {
    start + (stop - start) * amt
}

/// Wrap value to a range.
pub fn wrap(val: f32, mut from: f32, mut to: f32) -> f32 {
    if from > to { mem::swap(&mut from, &mut to); }
    let cycle = to - from;
    if cycle == 0.0 { return to; }
    val - cycle * ((val - from) / cycle).floor()
}

/// Floors value.
pub fn fast_floor<F: Float + FromPrimitive + ToPrimitive>(f: F) -> int {
    return if f > FromPrimitive::from_int(0).unwrap() {
        f.to_int().unwrap()
    } else {
        f.to_int().unwrap() - One::one()
    }
}

/// Implementation of grad1 for the ported _slang_library_noise1 method.
/// (This is used for `get_noise_walk` in signal).
pub fn grad1(hash: int, x: f32) -> f32 {
    let h: int = hash & 15;
    let mut grad: f32 = 1.0f32 + ((h & 7) as f32);
    if h & 8 > 0 { grad = (-1.0f32) * grad; }
    grad * x
}

/// Implementation of `perm` for the ported _slang_library_noise1 method
/// (This is used for `get_noise_walk` in signal).
static PERM : [u8, ..512] = [151u8, 160u8, 137u8, 91u8, 90u8, 15u8,
      131u8, 13u8, 201u8,95u8,96u8,53u8,194u8,233u8,7u8,225u8,140u8,36u8,103u8,30u8,69u8,142u8,8u8,99u8,37u8,240u8,21u8,10u8,23u8,
      190u8, 6u8,148u8,247u8,120u8,234u8,75u8,0u8,26u8,197u8,62u8,94u8,252u8,219u8,203u8,117u8,35u8,11u8,32u8,57u8,177u8,33u8,
      88u8,237u8,149u8,56u8,87u8,174u8,20u8,125u8,136u8,171u8,168u8, 68u8,175u8,74u8,165u8,71u8,134u8,139u8,48u8,27u8,166u8,
      77u8,146u8,158u8,231u8,83u8,111u8,229u8,122u8,60u8,211u8,133u8,230u8,220u8,105u8,92u8,41u8,55u8,46u8,245u8,40u8,244u8,
      102u8,143u8,54u8, 65u8,25u8,63u8,161u8, 1u8,216u8,80u8,73u8,209u8,76u8,132u8,187u8,208u8, 89u8,18u8,169u8,200u8,196u8,
      135u8,130u8,116u8,188u8,159u8,86u8,164u8,100u8,109u8,198u8,173u8,186u8, 3u8,64u8,52u8,217u8,226u8,250u8,124u8,123u8,
      5u8,202u8,38u8,147u8,118u8,126u8,255u8,82u8,85u8,212u8,207u8,206u8,59u8,227u8,47u8,16u8,58u8,17u8,182u8,189u8,28u8,42u8,
      223u8,183u8,170u8,213u8,119u8,248u8,152u8, 2u8,44u8,154u8,163u8, 70u8,221u8,153u8,101u8,155u8,167u8, 43u8,172u8,9u8,
      129u8,22u8,39u8,253u8, 19u8,98u8,108u8,110u8,79u8,113u8,224u8,232u8,178u8,185u8, 112u8,104u8,218u8,246u8,97u8,228u8,
      251u8,34u8,242u8,193u8,238u8,210u8,144u8,12u8,191u8,179u8,162u8,241u8, 81u8,51u8,145u8,235u8,249u8,14u8,239u8,107u8,
      49u8,192u8,214u8, 31u8,181u8,199u8,106u8,157u8,184u8, 84u8,204u8,176u8,115u8,121u8,50u8,45u8,127u8, 4u8,150u8,254u8,
      138u8,236u8,205u8,93u8,222u8,114u8,67u8,29u8,24u8,72u8,243u8,141u8,128u8,195u8,78u8,66u8,215u8,61u8,156u8,180u8,
      151u8,160u8,137u8,91u8,90u8,15u8,
      131u8,13u8,201u8,95u8,96u8,53u8,194u8,233u8,7u8,225u8,140u8,36u8,103u8,30u8,69u8,142u8,8u8,99u8,37u8,240u8,21u8,10u8,23u8,
      190u8, 6u8,148u8,247u8,120u8,234u8,75u8,0u8,26u8,197u8,62u8,94u8,252u8,219u8,203u8,117u8,35u8,11u8,32u8,57u8,177u8,33u8,
      88u8,237u8,149u8,56u8,87u8,174u8,20u8,125u8,136u8,171u8,168u8, 68u8,175u8,74u8,165u8,71u8,134u8,139u8,48u8,27u8,166u8,
      77u8,146u8,158u8,231u8,83u8,111u8,229u8,122u8,60u8,211u8,133u8,230u8,220u8,105u8,92u8,41u8,55u8,46u8,245u8,40u8,244u8,
      102u8,143u8,54u8, 65u8,25u8,63u8,161u8, 1u8,216u8,80u8,73u8,209u8,76u8,132u8,187u8,208u8, 89u8,18u8,169u8,200u8,196u8,
      135u8,130u8,116u8,188u8,159u8,86u8,164u8,100u8,109u8,198u8,173u8,186u8, 3u8,64u8,52u8,217u8,226u8,250u8,124u8,123u8,
      5u8,202u8,38u8,147u8,118u8,126u8,255u8,82u8,85u8,212u8,207u8,206u8,59u8,227u8,47u8,16u8,58u8,17u8,182u8,189u8,28u8,42u8,
      223u8,183u8,170u8,213u8,119u8,248u8,152u8, 2u8,44u8,154u8,163u8, 70u8,221u8,153u8,101u8,155u8,167u8, 43u8,172u8,9u8,
      129u8,22u8,39u8,253u8, 19u8,98u8,108u8,110u8,79u8,113u8,224u8,232u8,178u8,185u8, 112u8,104u8,218u8,246u8,97u8,228u8,
      251u8,34u8,242u8,193u8,238u8,210u8,144u8,12u8,191u8,179u8,162u8,241u8, 81u8,51u8,145u8,235u8,249u8,14u8,239u8,107u8,
      49u8,192u8,214u8, 31u8,181u8,199u8,106u8,157u8,184u8, 84u8,204u8,176u8,115u8,121u8,50u8,45u8,127u8, 4u8,150u8,254u8,
      138u8,236u8,205u8,93u8,222u8,114u8,67u8,29u8,24u8,72u8,243u8,141u8,128u8,195u8,78u8,66u8,215u8,61u8,156u8,180u8];

/// Implementation of perm lookup for the ported _slang_library_noise1 method
pub fn get_perm_val(i: uint) -> u8 { PERM[i] }


#[test]
pub fn test() {
    println!("Testing map - 0..10 -> -100.0..100.0");
    for i in range(0, 10) {
        print!("{}, ", map(i as f32, 0.0, 10.0, -100.0, 100.0));
    }
    println!("");

    println!("Testing wrap - wrapping 0..10 to 0..2.66");
    for i in range(0, 10) {
        print!("{}, ", wrap(i as f32, 0.0, 2.66));
    }
    println!("");

    println!("Testing fast_floor - for 0, 10 mapped to -2 to 2");
    for i in range(0.0f32, 10.0f32) {
        print!("{}, ", fast_floor(map(i as f32, 0.0f32, 10.0f32, -0.99f32, 0.99f32)));
    }
    println!("");

    println!("U8 PERM LIST! {} ", get_perm_val(30));
    
}

