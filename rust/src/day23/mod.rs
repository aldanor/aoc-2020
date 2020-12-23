use crate::utils::*;

#[inline]
fn parse_input(s: &[u8]) -> [u8; 9] {
    let mut x = [0; 9];
    assert_eq!(s.len(), 9);
    for i in 0..9 {
        x[i] = s.get_digit_at(i);
    }
    x
}

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(s: &[u8]) -> u64 {
    let mut x = parse_input(s);
    const N: usize = 9;
    let n = N as u8;
    for _ in 0..100 {
        let v = x[0];
        let (v1, v2, v3) = (x[1], x[2], x[3]); // labels of picked cups
        let mut z = v; // destination cup label
        while {
            z = n - (n - (z - 1)) % n;
            z == v1 || z == v2 || z == v3 // ensure it doesn't coincide with taken cups
        } {}
        let j = (0..N - 4).position(|j| x[j + 4] == z).unwrap(); // d/c *new* position
        for i in 0..=j {
            x[i] = x[i + 4]; // place the cups from next to picked up to including destination
        }
        for i in j + 5..N {
            x[i - 1] = x[i]; // place the cups at the end
        }
        x[j + 1] = v1;
        x[j + 2] = v2;
        x[j + 3] = v3;
        x[N - 1] = v;
    }
    let i = x.iter().position(|&x| x == 1).unwrap();
    x.rotate_left(i);
    x[1..]
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &x)| 10u64.pow(i as _) * (x as u64))
        .sum()
}

type Ix = u32;

fn make_list(head: impl AsRef<[u8]>, n: usize) -> Vec<Ix> {
    let head = head.as_ref().iter().map(|&x| x as Ix).collect::<Vec<_>>();
    let k = head.len();
    assert!(k > 0 && n >= k);
    let mut sorted = head.to_vec();
    sorted.sort_unstable();
    assert_eq!(sorted, (1..=k as Ix).collect::<Vec<_>>());

    let mut next = Vec::<Ix>::with_capacity(n + 1); // zero is an empty node
    unsafe { next.set_len(n + 1) };
    for i in 0..k - 1 {
        next[head[i] as usize] = head[i + 1] as Ix;
    }
    next[head[k - 1] as usize] = (k + 1) as Ix;
    for i in k + 1..=n - 1 {
        next[i as usize] = (i + 1) as Ix;
    }
    next[n as usize] = (head[0]) as Ix;
    next
}

#[inline]
pub fn part2(s: &[u8]) -> u64 {
    const N_CUPS: usize = 1_000_000;
    const N_MOVES: usize = 10_000_000;

    let x = parse_input(s);
    let mut next = make_list(&x, N_CUPS);
    let mut current = x[0] as usize;

    for _ in 0..N_MOVES {
        // find the three picked cups
        let x1 = next.get_at(current) as usize;
        let x2 = next.get_at(x1) as usize;
        let x3 = next.get_at(x2) as usize;

        // find the destination cup
        let mut dest = current - 1;
        if dest == 0 {
            dest = N_CUPS;
        }
        if dest == x1 || dest == x2 || dest == x3 {
            for _ in 0..3 {
                dest -= 1;
                if dest == 0 {
                    dest = N_CUPS;
                }
                if dest != x1 && dest != x2 && dest != x3 {
                    break;
                }
            }
        }

        // set the new current cup
        let node = next.get_at(x3);
        next.set_at(current, node);
        current = node as usize;

        // place the picked cups after the destination cup
        let node = next.get_at(dest);
        next.set_at(dest, x1 as _);
        next.set_at(x3, node);
    }
    (next[1] as u64) * (next[next[1] as usize] as u64)
}

#[test]
fn test_day23_part1() {
    assert_eq!(part1(input()), 89573246);
}

#[test]
fn test_day23_part2() {
    assert_eq!(part2(input()), 2029056128);
}
