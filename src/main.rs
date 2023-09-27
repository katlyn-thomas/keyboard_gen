#![allow(dead_code, unused)]

use std::{
    io::{self, Read, Write},
    fs::{File, self},
    collections::HashMap,
    path::PathBuf, str::FromStr, env, sync::{Arc, Mutex, atomic::AtomicUsize},
};

use rayon::prelude::*;
use rand::Rng;
use colored::Colorize;

#[derive(Debug, Clone)]
struct Keyboard {
    keys: [Key; 47],
    pub heatmap: [f32; 47],
    hands: [Finger; 8],
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum Finger {
    // row 0 (top row) through row 3 (bottom)
    LPinky([usize; 2], usize, usize, usize),
    LRing(usize, usize, usize, usize),
    LMid(usize, usize, usize, usize),
    LIndex([usize; 2], [usize; 2], [usize; 2], [usize; 2]),

    RIndex([usize; 2], [usize; 2], [usize; 2], [usize; 2]),
    RMid(usize, usize, usize, usize),
    RRing(usize, usize, usize, usize),
    RPinky([usize; 3], [usize; 4], [usize; 2], usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Key {
    Letter(char, char),
    StaticLetter(char, char),
    Number(char, char),
    Punctuation(char, char),
}

impl Key {
    pub fn match_char(&self, c: char) -> bool {
        match self {
            Key::Letter(k1, k2) =>       if *k1 == c || *k2 == c { true } else { false },
            Key::StaticLetter(k1, k2) => if *k1 == c || *k2 == c { true } else { false },
            Key::Number(k1, k2) =>       if *k1 == c || *k2 == c { true } else { false },
            Key::Punctuation(k1, k2) =>  if *k1 == c || *k2 == c { true } else { false },
        }
    }
    pub fn key_val(&self) -> String {
        match self {
            Key::Letter(x, _) => x.to_string(),
            Key::StaticLetter(x, _) => x.to_string(),
            Key::Punctuation(x, _) => x.to_string(),
            Key::Number(x, _) => x.to_string(),
        }
    }
}

impl PartialEq for Keyboard {
    fn eq(&self, other: &Self) -> bool {
        for (i, k) in self.keys.iter().enumerate() {
            if *k != other.keys[i] { return false }
        }
        true
    }

    fn ne(&self, other: &Self) -> bool {
        for (i, k) in self.keys.iter().enumerate() {
            if *k == other.keys[i] { return false }
        }
        true
        
    }
}

impl Keyboard {
    pub fn print_self(&self) {
        // `   1   2   3   4   5   6   7   8   9   0   -   =
        //   q   w   e   r   t   y   u   i   o   p   [   ]   \
        //    a   s   d   f   g   h   j   k   l   ;   '
        //      z   x   c   v   b   n   m   ,   .   /
        println!("{}   {}   {}   {}   {}   {}   {}   {}   {}   {}   {}   {}   {}",
            self.keys[0].key_val(),
            self.keys[1].key_val(),
            self.keys[2].key_val(),
            self.keys[3].key_val(),
            self.keys[4].key_val(),
            self.keys[5].key_val(),
            self.keys[6].key_val(),
            self.keys[7].key_val(),
            self.keys[8].key_val(),
            self.keys[9].key_val(),
            self.keys[10].key_val(),
            self.keys[11].key_val(),
            self.keys[12].key_val(),
        );
        println!("  {}   {}   {}   {}   {}   {}   {}   {}   {}   {}   {}   {}   {}",
            self.keys[13].key_val(),
            self.keys[14].key_val(),
            self.keys[15].key_val().red(),
            self.keys[16].key_val().red(),
            self.keys[17].key_val(),
            self.keys[18].key_val(),
            self.keys[19].key_val().green(),
            self.keys[20].key_val().green(),
            self.keys[21].key_val(),
            self.keys[22].key_val(),
            self.keys[23].key_val(),
            self.keys[24].key_val(),
            self.keys[25].key_val(),
        );
        println!("   {}   {}   {}   {}   {}   {}   {}   {}   {}   {}   {}",
            self.keys[26].key_val().red(),
            self.keys[27].key_val().red(),
            self.keys[28].key_val().red(),
            self.keys[29].key_val().red(),
            self.keys[30].key_val().red(),
            self.keys[31].key_val().red(),
            self.keys[32].key_val().red(),
            self.keys[33].key_val().red(),
            self.keys[34].key_val().red(),
            self.keys[35].key_val(),
            self.keys[36].key_val(),
        );
        println!("     {}   {}   {}   {}   {}   {}   {}   {}   {}   {}",
            self.keys[37].key_val(),
            self.keys[38].key_val(),
            self.keys[39].key_val().green(),
            self.keys[40].key_val().green(),
            self.keys[41].key_val(),
            self.keys[42].key_val(),
            self.keys[43].key_val().green(),
            self.keys[44].key_val().green(),
            self.keys[45].key_val(),
            self.keys[46].key_val(),
        );
    }
    pub fn reproduce(&self, mutations: usize) -> Keyboard {
        let mut new_keyboard = self.clone();

        let available_keys = [
            19, 20, 21, 22, 23, 24,
            35, 36, 39, 40, 41, 42, 43, 44, 45,
        ];

        let letter_only_keys = [
            19, 20, 39, 40, 43, 44
        ];

        let non_letter_only_keys = [
            21, 22, 23, 24,
            35, 36, 41, 42, 45,
        ];

        let letter_keys: Vec<usize> = available_keys
            .into_iter()
            .filter(|&k| {
                match new_keyboard.keys[k] {
                    Key::Letter(_, _) => true,
                    _ => false,
                }
            })
            .collect();

        let punc_keys: Vec<usize> = available_keys
            .into_iter()
            .filter(|&k| {
                match new_keyboard.keys[k] {
                    Key::Punctuation(_, _) => true,
                    _ => false,
                }
            })
            .collect();

        for _ in 0..mutations {
            let rand_key_index = available_keys[rand::thread_rng().gen_range(0..available_keys.len())];
            let rand_key_punc = punc_keys
                .clone()
                .into_iter()
                .find(|i| *i == rand_key_index)
                .is_some();
            let rand_key_letter_only = letter_only_keys
                .clone()
                .into_iter()
                .find(|i| *i == rand_key_index)
                .is_some();

            let other_key_index = match (rand_key_letter_only, rand_key_punc) {
                (true, _)      => letter_keys[rand::thread_rng().gen_range(0..letter_keys.len())],
                (false, true)  => non_letter_only_keys[rand::thread_rng().gen_range(0..non_letter_only_keys.len())],
                (false, false) => available_keys[rand::thread_rng().gen_range(0..available_keys.len())],
            };

            let key1 = new_keyboard.keys[rand_key_index].clone();
            let key2 = new_keyboard.keys[other_key_index].clone();

            {
                let key1_ref = new_keyboard.keys.get_mut(rand_key_index).unwrap();
                *key1_ref = key2;
            }
            {
                let key2_ref = new_keyboard.keys.get_mut(other_key_index).unwrap();
                *key2_ref = key1;
            }
        }

        new_keyboard
    }

    pub fn new_47() -> Self {
        use Finger::*;
        Self {
            keys: [
                Key::Punctuation('`', '~'),
                Key::Number('1', '!'),
                Key::Number('2', '@'),
                Key::Number('3', '#'),
                Key::Number('4', '$'),
                Key::Number('5', '%'),
                Key::Number('6', '^'),
                Key::Number('7', '&'),
                Key::Number('8', '*'),
                Key::Number('9', '('),
                Key::Number('0', ')'),
                Key::Punctuation(',', '<'),
                Key::Punctuation('.', '>'),

                Key::Punctuation('[', '{'),
                Key::Punctuation(']', '}'),
                Key::Letter('i', 'I'),
                Key::Letter('o', 'O'),
                Key::Punctuation('-', '_'),
                Key::Punctuation('=', '+'),
                Key::Letter('u', 'U'),
                Key::Letter('f', 'F'),
                Key::Letter('s', 'S'),
                Key::Letter('p', 'P'),
                Key::Letter('d', 'D'),
                Key::Letter('y', 'Y'),
                Key::Punctuation('\\', '|'),

                Key::Letter('a', 'A'),
                Key::Letter('r', 'R'),
                Key::Letter('t', 'T'),
                Key::Letter('e', 'E'),
                Key::Letter('c', 'c'),
                Key::StaticLetter('h', 'H'),
                Key::StaticLetter('j', 'J'),
                Key::StaticLetter('k', 'K'),
                Key::StaticLetter('l', 'L'),
                Key::Letter('z', 'Z'),
                Key::Letter('x', 'X'),
                
                Key::Punctuation(';', ':'),
                Key::Punctuation('\'', '"'),
                Key::Letter('g', 'G'),
                Key::Letter('v', 'V'),
                Key::Letter('b', 'B'),
                Key::Letter('n', 'N'),
                Key::Letter('m', 'M'),
                Key::Letter('q', 'Q'),
                Key::Letter('w', 'W'),
                Key::Punctuation('/', '?'),
            ],
            heatmap: [
                // `123456...
                3., 2., 2., 2., 2., 3., 3., 2., 2., 2., 2., 3., 4.,
                //qwerty...
                1.25, 1.5, 0.75, 0.75, 2., 2.5, 0.75, 0.75, 1., 1., 3., 3.5, 4.,
                //asdf...
                0.25, 0.5, 0., 0., 1., 1., 0., 0., 0.5, 0.25, 1.,
                // zxcv...
                1.25, 1.5, 1., 1., 2., 1., 1., 1., 1.5, 1.25,
            ],
            hands: [
                LPinky(
                    [0, 1],
                    13,
                    26,
                    37,
                ),
                LRing(
                    2,
                    14,
                    27,
                    38,
                ),
                LMid(
                    3,
                    15,
                    28,
                    39,
                ),
                LIndex(
                    [4, 5],
                    [16, 17],
                    [29, 30],
                    [40, 41],
                ),

                RIndex(
                    [6, 7],
                    [18, 19],
                    [31, 32],
                    [42, 43]
                ),
                RMid(
                    8,
                    20,
                    33,
                    44,
                ),
                RRing(
                    9,
                    21,
                    34,
                    45,
                ),
                RPinky(
                    [10, 11, 12],
                    [22, 23, 24, 25],
                    [35, 36],
                    46,
                ),
            ] 
        }
    }

    pub fn get_key(&self, c: char) -> Option<&Key> {
        let mut rv = None;
        for key in self.keys.iter() {
            if key.match_char(c) {
                rv = Some(key);
                break;
            }
        }
        rv
    }

    pub fn index_to_row(&self, index: usize) -> Option<usize> {
        if index < 13 { Some(0) }
        else if index >= 13 && index < 26 { Some(1) }
        else if index >= 26 && index < 37 { Some(2) }
        else if index >= 37 && index < 47 { Some(3) }
        else { None }
    }

    pub fn distance(&self, a: usize, b: usize) -> f64 {
        if a == b { return 0. };

        let a_hand = self.which_hand(a).unwrap();
        let b_hand = self.which_hand(b).unwrap();

        let a_row = self.index_to_row(a).unwrap();
        let b_row = self.index_to_row(b).unwrap();

        let row_diff = b_row as i8 - a_row as i8;

        if a_hand == b_hand {
            if row_diff == 0 { self.heatmap[b] as f64 * 0.75}
            else if row_diff < 0 { self.heatmap[b] as f64 * 1.5 }
            else { self.heatmap[b] as f64 }
        } else {
            self.heatmap[b] as f64 * 1.25
        }
    }

    pub fn char_to_index(&self, c: char) -> Option<usize> {
        for (i, k) in self.keys.iter().enumerate() {
            if k.match_char(c) {
                return Some(i.try_into().unwrap());
            }
        }
        None
    }

    pub fn which_hand(&self, index: usize) -> Option<Hand> {
        let mut index = 0_usize;
        loop {
            let hand = if index < 4 { Hand::Left } else { Hand::Right };
            if let Some(finger) = self.hands.get(index) {
                if finger.is_inside(index).is_some() { break Some(hand) }
                index += 1;
            } else {
                break None
            }
        }
    }
}

#[derive(PartialEq, Eq)]
enum Hand {
    Left,
    Right,
}

impl Finger {
    pub fn is_inside(&self, key: usize) -> Option<usize> {
        // returns the row if true
        match self {
            Self::LPinky(zero, one, two, three) => {
                let mut rv = None;
                for k in zero {
                    if *k == key { 
                        rv = Some(0)
                    }
                }
                if key == *one { rv = Some(1) }
                else if key == *two { rv = Some(2) }
                else if key == *three { rv = Some(3) }
                rv
            },
            Self::LRing(zero, one, two, three) => {
                if key == *zero { Some(1) }
                else if key == *one { Some(1) }
                else if key == *two { Some(2) }
                else if key == *three { Some(3) }
                else { None }
            },
            Self::LMid(zero, one, two, three) => {
                if key == *zero { Some(1) }
                else if key == *one { Some(1) }
                else if key == *two { Some(2) }
                else if key == *three { Some(3) }
                else { None }
            },
            Self::LIndex(zero, one, two, three) => {
                let mut rv = None;
                for k in zero {
                    if *k == key { 
                        rv = Some(0)
                    }
                }
                for k in one {
                    if *k == key { 
                        rv = Some(1)
                    }
                }
                for k in two {
                    if *k == key { 
                        rv = Some(2)
                    }
                }
                for k in three {
                    if *k == key { 
                        rv = Some(3)
                    }
                }
                rv
            },

            Self::RIndex(zero, one, two, three) => {
                let mut rv = None;
                for k in zero {
                    if *k == key { 
                        rv = Some(0)
                    }
                }
                for k in one {
                    if *k == key { 
                        rv = Some(1)
                    }
                }
                for k in two {
                    if *k == key {
                        rv = Some(2)
                    }
                }
                for k in three {
                    if *k == key { 
                        rv = Some(3)
                    }
                }
                rv
            },
            Self::RMid(zero, one, two, three) => {
                if key == *zero { Some(1) }
                else if key == *one { Some(1) }
                else if key == *two { Some(2) }
                else if key == *three { Some(3) }
                else { None }
            },
            Self::RRing(zero, one, two, three) => {
                if key == *zero { Some(1) }
                else if key == *one { Some(1) }
                else if key == *two { Some(2) }
                else if key == *three { Some(3) }
                else { None }
            },
            Self::RPinky(zero, one, two, three) => {
                let mut rv = None;
                for k in zero {
                    if *k == key { 
                        rv = Some(0)
                    }
                }
                for k in one {
                    if *k == key { 
                        rv = Some(1)
                    }
                }
                for k in two {
                    if *k == key { 
                        rv = Some(2)
                    }
                }
                if key == *three { rv = Some(3) }
                rv
            },
        }
    }
}


fn read_file(path: PathBuf) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}

fn read_dir(path: PathBuf, keyboard: &Keyboard, last_place: f64) -> io::Result<f64> {
    let mut score = 0_f64;

    for entry in fs::read_dir(path)? {
        if score > last_place { break }
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_file() {
            if let Ok(contents) = read_file(entry_path) {
                let mut last_char = ' ';
                contents.chars().for_each(|c| {
                    match (keyboard.char_to_index(last_char), keyboard.char_to_index(c)) {
                        (Some(a), Some(b)) => score += keyboard.distance(a, b),
                        (None, Some(b)) => score += keyboard.heatmap[b] as f64,
                        (_, None) => (),
                    }
                    last_char = c;
                })
            }
        } else if entry_path.is_dir() {
            score += read_dir(entry_path, keyboard, last_place).unwrap()
        }
    }

    Ok(score)
}

fn main() {
    let mut path = env::current_dir()
        .expect("lmao");
    path.push("pile");

    let mut keyboards: Vec<Keyboard> = vec![Keyboard::new_47(); 100]
        .iter()
        .enumerate()
        .map(|(i, k)| k.reproduce(match i % 4 {
                0 => 1,
                1 => 2,
                2 => 4,
                3 => 8,
                _ => panic!()
            }))
        .collect();
    let mut top_50 = vec![(100000000000., Keyboard::new_47()); 50];

    for x in 0..200 {
        println!("Starting Generation {}", x);
        println!();

        let count = AtomicUsize::new(0);
        let started = AtomicUsize::new(0);
        let last_place = top_50.iter().max_by(|&a, &b| a.0.partial_cmp(&b.0).unwrap()).unwrap().0;

        print!("\r0% complete");
        std::io::stdout().flush();

        let mut result = keyboards
            .par_iter()
            .enumerate()
            .map(|(i, keyboard)| {
                print!("\r{}% started - {}% complete",
                    if x == 0 {
                        started.load(std::sync::atomic::Ordering::Relaxed)
                    } else {
                        started.load(std::sync::atomic::Ordering::Relaxed) * 2
                    },
                    if x == 0 {
                        count.load(std::sync::atomic::Ordering::Relaxed)
                    } else {
                        count.load(std::sync::atomic::Ordering::Relaxed) * 2
                    },
                );
                std::io::stdout().flush();
                if let Some(entry) = top_50.iter().find(|(score, k_cmp)| *k_cmp == *keyboard) {
                    entry.clone()
                } else {
                    started.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let rv = (read_dir(path.clone(), &keyboard, last_place).expect("you fucked up"), keyboard.clone());
                    count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    rv
                }
            })
            .collect::<Vec<(f64, Keyboard)>>();

        print!("\r100% complete");
        std::io::stdout().flush();
        println!();
        println!();

        result.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        keyboards = result.iter().map(|(i, k)| k.clone()).collect();
        top_50 = result.iter()
            .enumerate()
            .filter_map(|(i, data)| {
                if i < 50 { Some(data.clone()) }
                else { None }
            })
            .collect();

        for (i, (score, keyboard)) in top_50.iter().enumerate() {
            let k = keyboards.get_mut(i + 50).unwrap();
            *k = keyboard.reproduce(match i % 4 {
                0 => 1,
                1 => 2,
                2 => 4,
                3 => 8,
                _ => panic!()
            })
        }

        let first = top_50.iter().min_by(|&a, &b| a.0.partial_cmp(&b.0).unwrap()).unwrap().0;
        let last = top_50.iter().max_by(|&a, &b| a.0.partial_cmp(&b.0).unwrap()).unwrap().0;
        let avg: f64 = top_50.iter().map(|key| key.0).sum::<f64>() / 50.;
        let med = top_50[25].0;

        println!("Score: {}", result[0].0);
        result[0].1.print_self();
        println!();

        println!("first place: {}", first);
        println!("50th place: {}", last);
        println!("average: {}", avg);
        println!("median (25th): {}", med);
        println!();
        println!("---------------------");
        println!();
    }
}
