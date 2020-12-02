pub fn input() -> Vec<i16> {
    include_str!("input.txt")
        .trim()
        .lines()
        .map(|l| l.parse().unwrap())
        .collect()
}

pub fn part1(s: &[i16]) -> u32 {
    let mut arr = [0i16; 4096];
    for &x in s {
        let x = x.min(2020) as usize;
        let rem = 2020 - x;
        unsafe { *arr.get_unchecked_mut(rem) = x as i16 };
        let y = unsafe { *arr.get_unchecked(x) };
        if y != 0 {
            return (x as u32) * (y as u32);
        }
    }
    0
}

#[inline]
fn find_sum_min2(s: &[i16]) -> i16 {
    let (mut a, mut b) = (i16::MAX, i16::MAX);
    for x in s {
        if *x >= b {
            // most likely
            continue;
        } else if *x >= a {
            // less likely
            b = *x;
        } else {
            // least likely
            b = a;
            a = *x;
        }
    }
    a + b
}

pub fn part2(s: &[i16]) -> u32 {
    let max = 2020 - find_sum_min2(s);
    let mut arr = Vec::with_capacity(s.len());
    for x in s {
        if *x <= max {
            arr.push(*x);
        }
    }
    quickersort::sort(&mut arr);
    let n = arr.len();

    for i in 0..n - 2 {
        let ai = unsafe { *arr.get_unchecked(i) };
        let ai_rem = ai - 2020;
        for j in i + 1..n - 1 {
            let aj = unsafe { *arr.get_unchecked(j) };
            let aij = ai_rem + aj;
            for k in j + 1..n {
                let ak = unsafe { *arr.get_unchecked(k) };
                let aijk = aij + ak;
                if aijk < 0 {
                    continue;
                } else if aijk > 0 {
                    break;
                } else {
                    return (ai as u32) * (aj as u32) * (ak as u32);
                }
            }
        }
    }
    0
}
