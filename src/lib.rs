use std::collections::HashMap;
use std::ops::Range;

use rayon::prelude::*;

pub struct FM {
	src: Vec<char>,
	/// "F" porition of LF map, stored as hashmap of characters to range in `p`
	f: HashMap<char, Range<usize>>,
	/// positions
	p: Vec<usize>,
	/// "L" porition of LF map, stored as indices into `p`
	l: Vec<usize>,
}

impl FM {
	pub fn build(src: &str) -> FM {
		let mut src: Vec<_> = src.chars().collect();
		src.push('\0');
		let n = src.len();

		let mut p: Vec<_> = (0..n).collect();
		p.par_sort_unstable_by(|&a, &b| {
			src.iter()
				.cycle()
				.skip(a)
				.take(n)
				.cmp(src.iter().cycle().skip(b).take(n))
		});

		let f: HashMap<_, _> = p
			.iter()
			.map(|i| src.get(*i).copied().unwrap_or('\0'))
			.collect::<Vec<_>>()
			.chunk_by(|a, b| a == b)
			.scan(0usize, |acc, chunk| {
				let index = *acc;
				*acc += chunk.len();
				Some((chunk[0], index..index + chunk.len()))
			})
			.collect();

		let mut charset = HashMap::<char, usize>::new();

		let l: Vec<_> = p
			.iter()
			.map(|pos| {
				if *pos == n {
					usize::MAX
				} else {
					let key = &src[(*pos + n - 1) % n];
					let rank = charset.entry(*key).or_insert(0);

					let index = f[key].start + *rank;
					*rank += 1;
					index
				}
			})
			.collect();

		FM { src, f, p, l }
	}

	pub fn search(&self, pattern: &str) -> Vec<usize> {
		self.search_for(pattern, None)
	}

	pub fn search_for(&self, pattern: &str, limit: Option<usize>) -> Vec<usize> {
		if pattern.is_empty() || !self.f.contains_key(&pattern.chars().nth(0).unwrap()) {
			return vec![];
		}

		let mut chars = pattern.chars().rev();
		let range = self.f[&chars.next().unwrap()].clone();

		let mut matches = Vec::<usize>::new();

		'outer: for mut i in range {
			for c in chars.clone() {
				let prev = (self.p[i] + self.src.len() - 1) % self.src.len();

				if self.src[prev] == c {
					i = self.l[i];
				} else {
					continue 'outer;
				}
			}

			matches.push(self.p[i]);

			if limit.map(|x| matches.len() >= x).unwrap_or(false) {
				break;
			}
		}

		matches
	}

	pub fn text(&self, range: Range<usize>) -> String {
		self.src[range].iter().collect()
	}
}
