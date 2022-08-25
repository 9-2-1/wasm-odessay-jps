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

fn sign_isize(x: isize) -> isize {
    return if x > 0 {
        1
    } else if x == 0 {
        0
    } else {
        -1
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
    fn signxy(&self) -> Self {
        pos!(sign_isize(self.x), sign_isize(self.y))
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
    frompos: Vec<Pos>,
    distance: Vec<isize>,
    openlist: BinaryHeap<Pointinfo>,
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
            frompos: Vec::with_capacity(siz),
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

    fn simphfunc(a: Pos, b: Pos) -> isize {
        let diff = a - b;
        return (((diff.x * diff.x + diff.y * diff.y) as f64).sqrt() * 100.0) as isize;
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

    fn point_add(&mut self, point: Pos, end: Pos, dist: isize, from: Pos) {
        return self.point_add_hval(point, dist, from, Self::hfunc(point, end));
    }

    fn point_add_simp(&mut self, point: Pos, end: Pos, dist: isize, from: Pos) {
        return self.point_add_hval(point, dist, from, Self::simphfunc(point, end));
    }

    fn point_add_hval(&mut self, point: Pos, dist: isize, from: Pos, hval: isize) {
        if self.can_walk(point) {
            let index = self.index(point);
            let dist_now = &mut self.distance[index];
            if *dist_now > dist {
                self.frompos[index] = from;
                *dist_now = dist;
                self.openlist.push(Pointinfo {
                    position: point,
                    distance: dist,
                    dist_gh: dist + hval,
                });
            }
        }
    }

    fn rushmove(&mut self, from: Pos, dist: isize, dir: Pos, end: Pos) -> bool {
        return self.rushmove_core(from, dist, dir, end, false);
    }

    fn rushmove_test(&mut self, from: Pos, dist: isize, dir: Pos, end: Pos) -> bool {
        return self.rushmove_core(from, dist, dir, end, true);
    }

    fn rushmove_core(&mut self, from: Pos, dist: isize, dir: Pos, end: Pos, testing: bool) -> bool {
        let mut pos = from + dir;
        let mut dist = dist + 2;
        loop {
            if !self.can_walk(pos) {
                return false;
            }
            let index = self.index(pos);
            if pos == end
                || self.distance[index] != isize::MAX
                || (!self.can_walk(pos - dir.flipxy()) && self.can_walk(pos - dir.flipxy() + dir))
                || (!self.can_walk(pos + dir.flipxy()) && self.can_walk(pos + dir.flipxy() + dir))
            {
                if !testing {
                    self.point_add(pos, end, dist, from);
                }
                return true;
            }
            pos = pos + dir;
            dist += 2;
        }
    }

    fn diagmove(&mut self, from: Pos, dist: isize, dir: Pos, end: Pos) -> bool {
        let mut pos = from + dir;
        let mut dist = dist + 3;
        loop {
            if !self.can_walk(pos) {
                return false;
            }
            let index = self.index(pos);
            if pos == end
                || self.distance[index] != isize::MAX
                || (!self.can_walk(pos - dir.xonly())
                    && self.can_walk(pos - dir.xonly() + dir.yonly()))
                || (!self.can_walk(pos - dir.yonly())
                    && self.can_walk(pos - dir.yonly() + dir.xonly()))
            {
                self.point_add(pos, end, dist, from);
                return true;
            }
            let turning = self.rushmove_test(pos, dist, dir.xonly(), end)
                || self.rushmove_test(pos, dist, dir.yonly(), end);
            if turning == true {
                self.point_add(pos, end, dist, from);
                return true;
            }
            pos = pos + dir;
            dist += 3;
        }
    }

    fn find(&mut self, begin: Pos, end: Pos) -> Vec<Pos> {
        let mut path = Vec::new();

        self.frompos = vec![pos!(-1, -1); self.map.len()];
        self.distance = vec![isize::MAX; self.map.len()];
        self.openlist.clear();

        self.point_add(end, begin, 0, end);
        while let Some(pinfo) = self.openlist.pop() {
            #[cfg(feature = "debug")]
            console_log(format!("{:?} <- {:?} ", pinfo, self.openlist).as_str());

            let pos = pinfo.position;
            let dist = pinfo.distance;

            if dist == self.distance[self.index(pos)] {
                if pos == begin {
                    break;
                }
                let index = self.index(pos);
                let dir = (pos - self.frompos[index]).signxy();
                if dir == pos!(0, 0) {
                    const RUSHDIR: [Pos; 4] = [pos!(1, 0), pos!(-1, 0), pos!(0, 1), pos!(0, -1)];
                    const DIAGDIR: [Pos; 4] = [pos!(1, 1), pos!(-1, 1), pos!(-1, -1), pos!(1, -1)];
                    for dir in RUSHDIR {
                        self.rushmove(pos, dist, dir, begin);
                    }
                    for dir in DIAGDIR {
                        self.diagmove(pos, dist, dir, begin);
                    }
                } else if dir.x == 0 || dir.y == 0 {
                    self.rushmove(pos, dist, dir, begin);
                    if !self.can_walk(pos - dir.flipxy()) && self.can_walk(pos - dir.flipxy() + dir)
                    {
                        self.diagmove(pos, dist, dir - dir.flipxy(), begin);
                    }
                    if !self.can_walk(pos + dir.flipxy()) && self.can_walk(pos + dir.flipxy() + dir)
                    {
                        self.diagmove(pos, dist, dir + dir.flipxy(), begin);
                    }
                } else {
                    self.rushmove(pos, dist, dir.xonly(), begin);
                    self.rushmove(pos, dist, dir.yonly(), begin);
                    self.diagmove(pos, dist, dir, begin);
                    if !self.can_walk(pos - dir.xonly())
                        && self.can_walk(pos - dir.xonly() + dir.yonly())
                    {
                        self.diagmove(pos, dist, dir.yonly() - dir.xonly(), begin);
                    }
                    if !self.can_walk(pos - dir.yonly())
                        && self.can_walk(pos - dir.yonly() + dir.xonly())
                    {
                        self.diagmove(pos, dist, dir.xonly() - dir.yonly(), begin);
                    }
                }
            }

            #[cfg(feature = "debug")]
            console_log(self.debug().as_str());
        }

        if self.distance[self.index(begin)] != isize::MAX {
            let mut find = begin;
            let mut cdir = pos!(0, 0);
            while end != find {
                let next = self.frompos[self.index(find)];
                let dir = find - next;

                // 如果连续三个点在同一条直线上，则不输出第二个点
                if cdir == pos!(0, 0) || dir.x * cdir.y - dir.y * cdir.x != 0 {
                    path.push(find);
                }

                find = next;
                cdir = dir;
            }
            path.push(find);
        }

        return path;
    }

    fn simplify(&mut self, path: &Vec<Pos>) -> Vec<Pos> {
        let mut simpath = Vec::new();
        let mut dir = pos!(0, 0);
        let mut begin = path[0];
        let mut shorten = false;
        if !path.is_empty() {
            let mut i = 0_usize;
            loop {
                let mut stop = false;
                if i == path.len() - 1 {
                    stop = true;
                } else {
                    let diff = path[i + 1] - path[i];
                    if diff.x != 0 {
                        if dir.x == 0 {
                            dir.x = diff.x;
                        } else if dir.x * diff.x < 0 {
                            // 判断符号不同
                            stop = true;
                        }
                    }
                    if diff.y != 0 {
                        if dir.y == 0 {
                            dir.y = diff.y;
                        } else if dir.y * diff.y < 0 {
                            // 判断符号不同
                            stop = true;
                        }
                    }
                }

                if stop {
                    if shorten {
                        #[cfg(feature = "debug")]
                        console_log(format!("Range: {:?} - {}", begin, i).as_str());

                        // simplify here
                        {
                            // let begin = path[i_begin];
                            let end = path[i];
                            let dir = (begin - end).signxy(); // 就是反过来的
                            if dir.x == 0 || dir.y == 0 {
                                simpath.push(begin);
                            } else {
                                self.frompos = vec![pos!(-1, -1); self.map.len()];
                                self.distance = vec![isize::MAX; self.map.len()];
                                self.openlist.clear();

                                let mut pointlist = Vec::new();

                                let mut cpos = end;
                                while cpos.y * dir.y <= begin.y * dir.y {
                                    cpos.x = end.x;
                                    while cpos.x * dir.x <= begin.x * dir.x {
                                        if self.can_walk(cpos)
                                            && self.can_walk(cpos + dir)
                                            && (!self.can_walk(cpos + dir.xonly())
                                                || !self.can_walk(cpos + dir.yonly()))
                                        {
                                            // 判断拐点
                                            pointlist.push(cpos);
                                        }
                                        cpos.x += dir.x;
                                    }
                                    cpos.y += dir.y;
                                }

                                pointlist.push(begin);
                                self.point_add_simp(end, begin, 0, end);

                                while let Some(pinfo) = self.openlist.pop() {
                                    #[cfg(feature = "debug")]
                                    console_log(
                                        format!("{:?} <- {:?} ", pinfo, self.openlist).as_str(),
                                    );

                                    let cpos = pinfo.position;
                                    if cpos == begin {
                                        break;
                                    }

                                    let dist = pinfo.distance;
                                    pointlist.iter().for_each(|pos| {
                                        let pos = *pos;
                                        let diff = pos - cpos;
                                        if diff.x * dir.x >= 0 && diff.y * dir.y >= 0 {
                                            let points = line_points(cpos, pos);
                                            let mut can_go = true;
                                            points.iter().for_each(|point| {
                                                if !self.can_walk(*point) {
                                                    can_go = false;
                                                }
                                            });
                                            if can_go {
                                                let dist2 = (((diff.x * diff.x + diff.y * diff.y)
                                                    as f64)
                                                    .sqrt()
                                                    * 100.0)
                                                    as isize;
                                                self.point_add_simp(pos, begin, dist + dist2, cpos);
                                            } else {
                                                let diff2 = pos - cpos;
                                                // 看拐过拐点之后能不能到
                                                let cpos2 = cpos + dir;
                                                let dist2 = dist + 141; // (2.0).sqrt()*100.0
                                                let points = line_points(cpos2, pos);
                                                let mut can_go = true;
                                                points.iter().for_each(|point| {
                                                    if !self.can_walk(*point) {
                                                        can_go = false;
                                                    }
                                                });
                                                if can_go {
                                                    let dist3 = (((diff2.x * diff2.x
                                                        + diff2.y * diff2.y)
                                                        as f64)
                                                        .sqrt()
                                                        * 100.0)
                                                        as isize;
                                                    // 中间节点，不需要被考虑
                                                    let index2 = self.index(cpos2);
                                                    self.frompos[index2] = cpos;
                                                    self.point_add_simp(
                                                        pos,
                                                        begin,
                                                        dist2 + dist3, /* (2.0).sqrt()*100.0 */
                                                        cpos2,
                                                    );
                                                }
                                            }
                                        }
                                    });

                                    #[cfg(feature = "debug")]
                                    console_log(self.debug().as_str());
                                }

                                let i = self.index(begin);
                                if self.distance[i] != isize::MAX {
                                    let mut find = begin;
                                    let mut cdir = pos!(0, 0);
                                    while end != find {
                                        let next = self.frompos[self.index(find)];
                                        let dir = find - next;

                                        // 如果连续三个点在同一条直线上，则不输出第二个点
                                        if cdir == pos!(0, 0)
                                            || dir.x * cdir.y - dir.y * cdir.x != 0
                                        {
                                            simpath.push(find);
                                        }

                                        find = next;
                                        cdir = dir;
                                    }
                                }
                            }
                            if i == path.len() - 1 {
                                break;
                            }
                            begin = simpath[simpath.len() - 1];
                        }
                        shorten = false;
                        dir = path[i] - begin;
                    } else {
                        begin = path[i];
                        dir = pos!(0, 0);
                    }
                } else {
                    shorten = true;
                    i += 1;
                }
            }
            simpath.push(path[path.len() - 1]);
        }

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
                let here = pos!(x, y);
                let i = self.index(here);
                dir += if self.map[i] == 0 {
                    if self.frompos[i] == pos!(-1, -1) {
                        "::"
                    } else {
                        let cdir = (here - self.frompos[i]).signxy();
                        DIRSYN[((cdir.y + 1) * 3 + (cdir.x + 2)) as usize]
                    }
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
                    let dir = ((dir.y + 1) * 3 + (dir.x + 2)) as i8;
                    map[index] = dir;
                }
            }
            let index = self.index(path[path.len() - 1]);
            map[index] = 5;
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

    #[cfg(feature = "debug")]
    {
        console_log(pathfinder.debug().as_str());
        console_log(pathfinder.debug_path(&path).as_str());
        console_log(format!("{:?}\n", path).as_str());
    }

    let smoothpath = pathfinder.simplify(&path);

    #[cfg(feature = "debug")]
    {
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
