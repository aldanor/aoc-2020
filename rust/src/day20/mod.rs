use std::mem;

use crate::utils::*;

use arrayvec::ArrayVec;

const WIDTH: usize = 10;
const SIDE: usize = 12;
const N_TILES: usize = SIDE * SIDE;
const N: usize = 256; // power of 2 >= N_TILES
const MAX_EDGES: usize = 1 << WIDTH;
const W: usize = WIDTH - 2; // tile width in the bitmap
const N_PIXELS: usize = SIDE * W; // pixel width of final bitmap

type Edge = u16;
type Rotation = u8;
type TileNum = usize;
type EdgeMap = ArrayVec<[ArrayVec<[(TileNum, Rotation); 2]>; MAX_EDGES]>;
type Image = [[(TileNum, Rotation); SIDE]; SIDE];
type Bitmap = [bool; N_PIXELS * N_PIXELS];

#[inline]
fn flip_edge(edge: Edge) -> Edge {
    edge.reverse_bits() >> (8 * mem::size_of::<Edge>() - WIDTH)
}

#[derive(Debug, Clone, Copy, Default)]
struct Tile {
    id: u16,
    edges: [u16; 8], // edges 0 2 4 6 are normal, 1 3 5 7 are flipped
}

impl Tile {
    #[inline]
    pub fn parse(s: &mut &[u8]) -> Self {
        *s = s.advance(5);
        let id = parse_int_fast(s, 4, 4);
        *s = s.advance(1);

        let (mut up, mut right, mut down, mut left) = (0, 0, 0, 0);
        // up
        let top_row = *s;
        for i in 0..WIDTH {
            up |= ((top_row.get_at(i) == b'#') as u16) << i;
        }
        // down
        let bottom_row = (*s).advance((WIDTH - 1) * (WIDTH + 1));
        for i in 0..WIDTH {
            down |= ((bottom_row.get_at(i) == b'#') as u16) << (WIDTH - 1 - i);
        }
        // right
        let right_column = (*s).advance(WIDTH - 1);
        for i in 0..WIDTH {
            right |= ((right_column.get_at(i * (WIDTH + 1)) == b'#') as u16) << i;
        }
        // left
        let left_column = *s;
        for i in 0..WIDTH {
            left |= ((left_column.get_at(i * (WIDTH + 1)) == b'#') as u16) << (WIDTH - 1 - i);
        }

        *s = s.advance(WIDTH * (WIDTH + 1) + 1);

        // to turn: index = (index + 2 * angle) % 8 // (each turn is 90 deg; can be negative)
        // to flip: index = 7 - index
        let edges = [
            up,
            flip_edge(left),
            right,
            flip_edge(down),
            down,
            flip_edge(right),
            left,
            flip_edge(up),
        ];
        Self { id, edges }
    }
}

#[inline]
fn parse_tiles(mut s: &[u8]) -> ArrayVec<[Tile; N]> {
    let mut tiles = ArrayVec::new();
    for _ in 0..N_TILES {
        tiles.push(Tile::parse(&mut s));
    }
    assert_eq!(s.len(), 0);
    tiles
}

#[inline]
fn get_bitmap_offsets(width: usize, height: usize) -> [(isize, isize, usize); 8] {
    // i-th element is if rotation #i faces top; tuples are of the form (offset, dx, dy)
    const K: isize = N_PIXELS as _;
    [
        (1, K, 0 + 0),
        (K, 1, 0 + 0),
        (-K, 1, (width - 1) * N_PIXELS + 0),
        (1, -K, (height - 1) * N_PIXELS + 0),
        (-1, -K, (height - 1) * N_PIXELS + (width - 1)),
        (-K, -1, (width - 1) * N_PIXELS + (height - 1)),
        (K, -1, 0 + (height - 1)),
        (-1, K, 0 + (width - 1)),
    ]
}

#[inline]
fn parse_bitmap(mut s: &[u8], image: &Image) -> Bitmap {
    let mut coords = [(0, 0, 0); N_TILES];
    for i in 0..SIDE {
        for j in 0..SIDE {
            coords[image[i][j].0] = (i, j, image[i][j].1);
        }
    }

    let offsets = get_bitmap_offsets(W, W);
    let mut bitmap = [false; N_PIXELS * N_PIXELS];

    for &(y, x, rotation) in &coords {
        let (dx, dy, offset) = offsets[rotation as usize];
        let mut offset = (offset + y * W * N_PIXELS + x * W) as isize;
        s = s.advance(WIDTH + 13);
        for _ in 0..W {
            let mut local_offset = offset;
            for i in 0..W {
                bitmap[local_offset as usize] = s.get_at(i) == b'#';
                local_offset += dx;
            }
            s = s.advance(WIDTH + 1);
            offset += dy;
        }
        s = s.advance(WIDTH + 1);
    }
    bitmap
}

#[inline]
fn build_edge_map(tiles: &[Tile]) -> EdgeMap {
    // for each of the 1024 edges, track the list of tiles that have it along with rotation
    let mut edge_map = EdgeMap::new();
    for _ in 0..MAX_EDGES {
        edge_map.push(Default::default());
    }
    assert_eq!(tiles.len(), N_TILES);
    for (tile_num, tile) in tiles.iter().enumerate() {
        for (rotation, &edge) in tile.edges.iter().enumerate() {
            edge_map[edge as usize].push((tile_num, rotation as _));
        }
    }
    edge_map
}

#[inline]
fn find_boundary(edge_map: &EdgeMap) -> [usize; 4 * (SIDE - 1)] {
    let mut edge_counts = [0u8; N_TILES]; // number of boundary edges for each tile
    for edge_parents in edge_map {
        if edge_parents.len() == 1 {
            let tile_num = edge_parents[0].0;
            edge_counts[tile_num] += 1;
        }
    }
    let mut boundary = [0; 4 * (SIDE - 1)];
    let (mut n_corners, mut n_edges) = (0, 0);
    for (tile_num, &n) in edge_counts.iter().enumerate() {
        if n == 2 {
            boundary[4 + n_edges] = tile_num;
            n_edges += 1;
        } else if n == 4 {
            boundary[n_corners] = tile_num;
            n_corners += 1;
        }
    }
    // first 4 tiles returned are corners, the rest are edges
    assert_eq!(n_corners, 4);
    assert_eq!(n_edges, 4 * (SIDE - 2));
    boundary
}

#[inline]
fn build_image(tiles: &[Tile], edge_map: &EdgeMap) -> Image {
    // rotation = which edge faces top (this already accounts for flipping)
    let mut image: Image = [[(0, 0); SIDE]; SIDE];

    // place top-left tile (pick any corner)
    let tile_num = find_boundary(&edge_map)[0];
    let is_boundary = |i| edge_map[tiles[tile_num].edges[(i as usize) % 8] as usize].len() == 1;
    let rotation = (0..8)
        .filter(|&i| is_boundary(i) && is_boundary(i + 6))
        .next()
        .unwrap();
    image[0][0] = (tile_num, rotation);

    // place the top row (link from left to right)
    for j in 1..SIDE {
        let (left_tile_num, left_rotation) = image[0][j - 1];
        // find the right edge: r + 2, each version matches a flipped version of itself: 7 - r
        let edge = tiles[left_tile_num].edges[7 - (left_rotation as usize + 2) % 8 as usize];
        let parents = &edge_map[edge as usize];
        let (tile_num, rotation) = parents[(parents[0].0 == left_tile_num) as usize];
        image[0][j] = (tile_num, (rotation + 2) % 8); // record the top edge as rotation: r + 2
    }

    // place all other rows below (link each tile to the one above)
    for i in 1..SIDE {
        for j in 0..SIDE {
            let (top_tile_num, top_rotation) = image[i - 1][j];
            let edge = tiles[top_tile_num].edges[7 - (top_rotation as usize + 4) % 8 as usize];
            let parents = &edge_map[edge as usize];
            image[i][j] = parents[(parents[0].0 == top_tile_num) as usize];
        }
    }

    // check that we haven't messed anything up
    if cfg!(debug_assertions) {
        check_image_for_correctness(&tiles, &edge_map, &image);
    }

    image
}

#[inline]
fn check_image_for_correctness(tiles: &[Tile], edge_map: &EdgeMap, image: &Image) {
    // check that all tiles are used and exactly once
    let mut tile_nums: ArrayVec<[_; N]> = ArrayVec::new();
    for i in 0..SIDE {
        for j in 0..SIDE {
            tile_nums.push(image[i][j].0);
        }
    }
    tile_nums.sort_unstable();
    for i in 0..N_TILES {
        assert_eq!(tile_nums[i], i);
    }
    // check all boundary sides and corners for correctness
    let n_parents = |i: usize, j: usize, rotate: usize| {
        edge_map[tiles[image[i][j].0].edges[((image[i][j].1 as usize) + rotate) % 8] as usize].len()
    };
    for i in 0..SIDE {
        assert_eq!(n_parents(0, i, 0), 1); // top
        assert_eq!(n_parents(i, SIDE - 1, 2), 1); // right
        assert_eq!(n_parents(SIDE - 1, i, 4), 1); // bottom
        assert_eq!(n_parents(i, 0, 6), 1); // left
    }
    // check all interior tiles for correctness
    for i in 1..SIDE - 1 {
        for j in 1..SIDE - 1 {
            let (tile1, rot1) = (tiles[image[i][j].0], image[i][j].1);
            for direction in 0..4 {
                let index = match direction {
                    0 => (i - 1, j),
                    1 => (i, j + 1),
                    2 => (i + 1, j),
                    _ => (i, j - 1),
                };
                let (tile2, rot2) = (tiles[image[index.0][index.1].0], image[index.0][index.1].1);
                let edge1 = tile1.edges[((rot1 + direction * 2) % 8) as usize];
                let edge2 = tile2.edges[((rot2 + (direction + 2) * 2) % 8) as usize];
                assert_eq!(edge1, flip_edge(edge2));
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
struct Mask {
    offsets: ArrayVec<[usize; 64]>,
    width: usize,
    height: usize,
}

#[inline]
fn get_monster_masks() -> [Mask; 8] {
    let sea_monster_repr = [
        b"                  # ",
        b"#    ##    ##    ###",
        b" #  #  #  #  #  #   ",
    ];

    let width = sea_monster_repr[0].len();
    let height = sea_monster_repr.len();
    let offsets = get_bitmap_offsets(width, height);
    let mut masks: [Mask; 8] = Default::default();

    for rotation in 0..8 {
        let mask = &mut masks[rotation];
        if [0, 4, 7, 3].contains(&rotation) {
            mask.width = width;
            mask.height = height;
        } else {
            mask.width = height;
            mask.height = width;
        }
        let (dx, dy, offset) = offsets[rotation];
        let mut offset = offset as isize;
        for y in 0..height {
            let mut local_offset = offset;
            for x in 0..width {
                if sea_monster_repr[y][x] == b'#' {
                    mask.offsets.push(local_offset as _);
                }
                local_offset += dx;
            }
            offset += dy;
        }
    }

    masks
}

#[derive(Debug, Copy, Clone)]
struct Cursor<T> {
    ptr: *const T,
}

impl<T: Copy> Cursor<T> {
    #[inline(always)]
    pub fn step(&mut self) {
        self.jump(1);
    }

    #[inline(always)]
    pub fn jump(&mut self, n: usize) {
        unsafe { self.ptr = self.ptr.add(n) };
    }

    #[inline(always)]
    pub fn get(&mut self, n: usize) -> T {
        unsafe { *self.ptr.add(n) }
    }
}

impl Cursor<bool> {
    #[inline(always)]
    pub fn check_mask(&mut self, offsets: &[usize]) -> bool {
        for &offset in offsets {
            if !self.get(offset) {
                return false;
            }
        }
        return true;
    }
}

#[inline]
fn count_monsters(bitmap: &Bitmap, masks: &[Mask; 8]) -> u16 {
    let (mut x0, mut y0, mut rotation) = (0, 0, 0);
    let mut cursor = Cursor {
        ptr: bitmap.as_ptr(),
    };

    // first, we need to figure mask rotation - find just one matching instance
    'outer: for y in 0..N_PIXELS {
        for x in 0..N_PIXELS {
            for (i, mask) in masks.iter().enumerate() {
                if x < N_PIXELS - mask.width && y < N_PIXELS - mask.height {
                    if cursor.check_mask(&mask.offsets) {
                        x0 = x;
                        y0 = y;
                        rotation = i;
                        cursor.step();
                        break 'outer;
                    }
                }
            }
            cursor.step();
        }
    }

    let mut count = 1;
    let mask = &masks[rotation];

    // finish the unfinished row
    for _ in (x0 + 1)..(N_PIXELS - mask.width) {
        if cursor.check_mask(&mask.offsets) {
            count += 1;
        }
        cursor.step();
    }
    cursor.jump(mask.width);

    // now finish all other rows
    for _ in (y0 + 1)..(N_PIXELS - mask.height) {
        for _ in 0..(N_PIXELS - mask.width) {
            if cursor.check_mask(&mask.offsets) {
                count += 1;
            }
            cursor.step();
        }
        cursor.jump(mask.width);
    }

    count
}

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(s: &[u8]) -> u64 {
    let tiles = parse_tiles(s);
    let edge_map = build_edge_map(&tiles);
    find_boundary(&edge_map)
        .iter()
        .take(4)
        .map(|&id| tiles[id].id as u64)
        .product()
}

#[inline]
pub fn part2(s: &[u8]) -> u16 {
    let tiles = parse_tiles(s);
    let edge_map = build_edge_map(&tiles);
    let image = build_image(&tiles, &edge_map);
    let bitmap = parse_bitmap(s, &image);
    let masks = get_monster_masks();
    let n_monsters = count_monsters(&bitmap, &masks);
    bitmap.iter().map(|&x| x as u16).sum::<u16>() - (masks[0].offsets.len() as u16 * n_monsters)
}

#[test]
fn test_day20_part1() {
    assert_eq!(part1(input()), 14129524957217);
}

#[test]
fn test_day20_part2() {
    assert_eq!(part2(input()), 1649);
}
