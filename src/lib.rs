use std::cmp;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt;
use std::fmt::Debug;
use std::ops::{Add, Sub};

use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(feature = "debug")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn console_log(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn console_log_i32(s: i32);
}

#[derive(Clone, Copy)]
struct Pointinfo {
    position: Pos,
    distance: isize,
    dist_gh: isize,
}

impl PartialEq for Pointinfo {
    fn eq(&self, other: &Self) -> bool {
        other.dist_gh.eq(&self.dist_gh)
    }
}

impl Eq for Pointinfo {}

impl PartialOrd for Pointinfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Pointinfo {
    fn cmp(&self, other: &Self) -> Ordering {
        other.dist_gh.cmp(&self.dist_gh)
    }
}

impl Debug for Pointinfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?}={:?}~{:?}",
            self.position, self.distance, self.dist_gh
        )
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Pos {
    x: isize,
    y: isize,
}

impl Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

macro_rules! pos {
    ($x:expr,$y:expr) => {
        Pos { x: $x, y: $y }
    };
}

impl Pos {
    fn xonly(&self) -> Self {
        pos!(self.x, 0)
    }
    fn yonly(&self) -> Self {
        pos!(0, self.y)
    }
    fn flipxy(&self) -> Self {
        pos!(self.y, self.x)
    }
}

impl Add for Pos {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        pos!(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Pos {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        pos!(self.x - other.x, self.y - other.y)
    }
}

#[derive(Debug)]
struct AStarJPS<'a> {
    size: Pos,
    map: &'a [u8],
    direction: Vec<i8>,
    distance: Vec<isize>,
    openlist: BinaryHeap<Pointinfo>,
}

macro_rules! dir {
    ($v:expr) => {{
        let dir: isize = (($v) as isize) - 1;
        pos!(dir % 3 - 1, dir / 3 - 1)
    }};
}

macro_rules! dir_i8 {
    ($d:expr) => {{
        (($d.y + 1) * 3 + ($d.x + 1) + 1) as i8
    }};
}

fn roundidiv(a: isize, b: isize) -> isize {
    return if a > 0 {
        (2 * a + b) / (2 * b) // round(a/b) = floor(a/b+1/2) = (2*a+b)/(2*b)
    } else {
        (2 * a - b + 1) / (2 * b) // round(a/b) = ceil'(a/b-1/2) = (2*a-b+1)/(2*b)
    };
}

fn line_points(a: Pos, b: Pos) -> Vec<Pos> {
    let mut resu = Vec::new();
    let step = cmp::max((a.x - b.x).abs(), (a.y - b.y).abs());
    for i in 0..step {
        resu.push(pos![
            a.x + roundidiv((b.x - a.x) * i, step),
            a.y + roundidiv((b.y - a.y) * i, step)
        ]);
    }
    resu.push(b);

    //#[cfg(feature = "debug")]
    //console_log(format!("{:?} {:?} {:?}", a, b, resu).as_str());

    return resu;
}

impl<'a> AStarJPS<'a> {
    fn new(size: Pos, map: &'a [u8]) -> Self {
        let siz = (size.x * size.y) as usize;
        return Self {
            size: size,
            map: map,
            direction: Vec::with_capacity(siz),
            distance: Vec::with_capacity(siz),
            openlist: BinaryHeap::new(),
        };
    }

    fn hfunc(a: Pos, b: Pos) -> isize {
        let diff = a - b;
        return if diff.x.abs() < diff.y.abs() {
            diff.x.abs() + 2 * diff.y.abs()
        } else {
            2 * diff.x.abs() + diff.y.abs()
        };
    }

    fn index(&self, point: Pos) -> usize {
        return (point.y * self.size.x + point.x) as usize;
    }

    fn can_walk(&self, point: Pos) -> bool {
        return point.x >= 0
            && point.x < self.size.x
            && point.y >= 0
            && point.y < self.size.y
            && self.map[self.index(point)] == 0;
    }

    fn point_add(&mut self, point: Pos, end: Pos, dist: isize, dir: Pos) {
        if self.can_walk(point) {
            let index = self.index(point);
            let dist_now = &mut self.distance[index];
            if *dist_now > dist {
                self.direction[index] = dir_i8!(dir);
                *dist_now = dist;
                self.openlist.push(Pointinfo {
                    position: point,
                    distance: dist,
                    dist_gh: dist + Self::hfunc(point, end),
                });
            }
        }
    }

    fn rushmove(&mut self, pinfo: &Pointinfo, dir: Pos, end: Pos) -> bool {
        let Pointinfo {
            position: mut pos,
            dist_gh: _dist_gh, // do not use
            distance: mut dist,
        } = pinfo;
        pos = pos + dir;
        dist += 2;
        loop {
            if !self.can_walk(pos) {
                return false;
            }
            let index = self.index(pos);
            if self.distance[index] == -1 {
                return false;
            }
            if pos == end
                || (!self.can_walk(pos - dir.flipxy()) && self.can_walk(pos - dir.flipxy() + dir))
                || (!self.can_walk(pos + dir.flipxy()) && self.can_walk(pos + dir.flipxy() + dir))
            {
                self.point_add(pos, end, dist, dir);
                return true;
            }
            self.distance[index] = -1;
            pos = pos + dir;
            dist += 2;
        }
    }

    fn diagmove(&mut self, pinfo: &Pointinfo, dir: Pos, end: Pos) -> bool {
        let Pointinfo {
            position: mut pos,
            dist_gh: _dist_gh, // do not use
            distance: mut dist,
        } = pinfo;
        pos = pos + dir;
        dist += 3;
        loop {
            if !self.can_walk(pos) {
                return false;
            }
            let index = self.index(pos);
            if self.distance[index] == -1 {
                return false;
            }
            let turning = self.rushmove(
                &Pointinfo {
                    position: pos,
                    dist_gh: 0, // fake
                    distance: dist,
                },
                dir.xonly(),
                end,
            ) || self.rushmove(
                &Pointinfo {
                    position: pos,
                    dist_gh: 0, // fake
                    distance: dist,
                },
                dir.yonly(),
                end,
            );
            if pos == end
                || (!self.can_walk(pos - dir.xonly())
                    && self.can_walk(pos - dir.xonly() + dir.yonly()))
                || (!self.can_walk(pos - dir.yonly())
                    && self.can_walk(pos - dir.yonly() + dir.xonly()))
            {
                self.point_add(pos, end, dist, dir);
                return true;
            }
            if turning == true {
                self.point_add(pos, end, dist, dir);
                return true;
            }
            self.distance[index] = -1;
            pos = pos + dir;
            dist += 3;
        }
    }

    fn find(&mut self, begin: Pos, end: Pos) -> Vec<Pos> {
        self.direction.clear();
        self.distance.clear();
        self.openlist.clear();

        let mut path = Vec::new();

        for _x in 0..self.map.len() {
            self.direction.push(0_i8);
            self.distance.push(isize::MAX);
        }

        self.point_add(end, begin, 0, pos!(0, 0));
        while let Some(pinfo) = self.openlist.pop() {
            #[cfg(feature = "debug")]
            console_log(format!("{:?} <- {:?} ", pinfo, self.openlist).as_str());

            if pinfo.distance == self.distance[self.index(pinfo.position)] {
                if pinfo.position == begin {
                    break;
                }
                const RUSHDIR: [Pos; 4] = [pos!(1, 0), pos!(-1, 0), pos!(0, 1), pos!(0, -1)];
                const DIAGDIR: [Pos; 4] = [pos!(1, 1), pos!(-1, 1), pos!(-1, -1), pos!(1, -1)];
                for dir in RUSHDIR {
                    self.rushmove(&pinfo, dir, begin);
                }
                for dir in DIAGDIR {
                    self.diagmove(&pinfo, dir, begin);
                }
            }

            #[cfg(feature = "debug")]
            console_log(self.debug().as_str());
        }

        //console_log(self.debug().as_str());

        if self.distance[self.index(begin)] != isize::MAX {
            let mut find = begin;
            let mut cdir = 0;
            while end != find {
                let dir = self.direction[self.index(find)];
                if dir != 0 && dir != cdir {
                    cdir = dir;
                    path.push(find);
                }
                find = find - dir!(cdir);
                if !self.can_walk(find) {
                    #[cfg(feature = "debug")]
                    {
                        console_log(format!("{:?}", self).as_str());
                        console_log(format!("{:?}", find).as_str());
                    }

                    break;
                }
            }
            path.push(find);
        }

        return path;
    }

    fn simplify(&self, path: &Vec<Pos>) -> Vec<Pos> {
        let mut simpath = Vec::new();
        simpath.push(pos!(0, 0));
        return simpath;
    }

    #[cfg(feature = "debug")]
    fn debug(&self) -> String {
        let mut dir = String::new();
        let mut dis = String::new();

        const DIRSYN: [&'static str; 10] =
            ["::", "JJ", "vv", "LL", ">>", "88", "<<", "77", "^^", "rr"];

        for y in 0..self.size.y {
            for x in 0..self.size.x {
                let i = self.index(pos!(x, y));
                dir += if self.map[i] == 0 {
                    DIRSYN[self.direction[i] as usize]
                } else {
                    "  "
                };
                if self.distance[i] == isize::MAX {
                    dis += "  ";
                } else if self.distance[i] == -1 {
                    dis += "::";
                } else {
                    let dstr = (100 + self.distance[i]).to_string();
                    dis += &dstr[(dstr.len() - 2)..dstr.len()];
                }
            }
            dir += "\n";
            dis += "\n";
        }

        return format!("{}\n{}\n", dir, dis);
    }

    #[cfg(feature = "debug")]
    fn debug_path(&self, path: &Vec<Pos>) -> String {
        let mut dir = String::new();
        let mut map = self.map.iter().map(|x| (x * 10) as i8).collect::<Vec<_>>();

        if !path.is_empty() {
            // 这里要先判path是否为空，
            // 否则path.len() - 1作为无符号整数usize会向下溢出
            for i in 0..(path.len() - 1) {
                let lpath = line_points(path[i], path[i + 1]);
                for i in 0..(lpath.len() - 1) {
                    let index = self.index(lpath[i]);
                    let dir = lpath[i] - lpath[i + 1];
                    let dir = dir_i8!(dir);
                    map[index] = dir;
                }
            }
            let index = self.index(path[path.len() - 1]);
            map[index] = dir_i8!(pos!(0, 0));
        }

        const DIRSYN: [&'static str; 11] = [
            "::", "JJ", "vv", "LL", ">>", "88", "<<", "77", "^^", "rr", "##",
        ];

        for y in 0..self.size.y {
            for x in 0..self.size.x {
                let i = self.index(pos!(x, y));
                dir += DIRSYN[map[i] as usize];
            }
            dir += "\n";
        }

        return dir;
    }
}

#[wasm_bindgen]
pub fn a_star_jps(
    map: &mut [u8],
    map_x: isize,
    map_y: isize,
    begin_x: isize,
    begin_y: isize,
    end_x: isize,
    end_y: isize,
) -> Vec<isize> {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let mut pathfinder = AStarJPS::new(pos!(map_x, map_y), map);
    let path = pathfinder.find(pos!(begin_x, begin_y), pos!(end_x, end_y));
    let smoothpath = pathfinder.simplify(&path);

    #[cfg(feature = "debug")]
    {
        console_log(pathfinder.debug().as_str());
        console_log(pathfinder.debug_path(&path).as_str());
        console_log(format!("{:?}\n", path).as_str());
        console_log(pathfinder.debug_path(&smoothpath).as_str());
        console_log(format!("{:?}\n", smoothpath).as_str());
    }

    let mut resu = Vec::with_capacity((path.len() + smoothpath.len()) * 2 + 1);

    resu.push(path.len() as isize);

    path.iter().for_each(|point| {
        resu.push(point.x);
        resu.push(point.y);
    });

    smoothpath.iter().for_each(|point| {
        resu.push(point.x);
        resu.push(point.y);
    });

    return resu;
}
