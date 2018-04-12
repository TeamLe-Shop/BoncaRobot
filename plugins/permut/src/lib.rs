#[macro_use]
extern crate plugin_api;

use plugin_api::prelude::*;

struct PermutPlugin;

impl PermutPlugin {
    fn permut(_this: &mut Plugin, arg: &str, ctx: Context) {
        let perms = permutations(arg);
        let mut msg = String::new();
        for p in perms {
            msg += &(p + ", ");
            if msg.len() > 700 {
                let cut = String::from_utf8_lossy(&msg.as_bytes()[..700]);
                ctx.send_channel(&format!(
                    "{}: {}",
                    ctx.sender.nickname(),
                    &format!("{}...", cut)
                ));
                return;
            }
        }
        ctx.send_channel(&format!("{}: {}", ctx.sender.nickname(), &msg));
    }
}

impl Plugin for PermutPlugin {
    fn new() -> Self {
        PermutPlugin
    }
    fn register(&self, meta: &mut PluginMeta) {
        meta.command("permut", "permutate shit", Self::permut);
    }
}

plugin_export!(PermutPlugin);

fn permutations(word: &str) -> Permutations {
    let mut chars = word.chars().collect::<Vec<char>>();
    chars.sort();
    Permutations {
        chars: chars.into_boxed_slice(),
        exhausted: false,
    }
}

struct Permutations {
    // Characters, sorted
    chars: Box<[char]>,
    // If this is true, we've exhausted all possible permutation, so return `None`.
    exhausted: bool,
}

impl Iterator for Permutations {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        let ret = if self.exhausted {
            None
        } else {
            Some(self.chars.iter().collect())
        };
        self.advance();
        ret
    }
}

impl Permutations {
    /// Advance to the next permutation.
    fn advance(&mut self) {
        let len = self.chars.len();
        if len < 2 {
            self.exhausted = true;
            return;
        }
        let (left, right) = self.find_swappees();
        self.chars.swap(left, right);
        self.chars[left + 1..].sort();
    }
    // Left is the index of the first value that's smaller than something to the right.
    // Right is the index of the smallest value that's bigger than [left].
    fn find_swappees(&mut self) -> (usize, usize) {
        let len = self.chars.len();
        let mut left_idx = len - 2;
        loop {
            match self.find_smallest_value_to_the_right_bigger_than_value_at_idx(left_idx) {
                Some(idx) => return (left_idx, idx),
                None => {
                    if left_idx == 0 {
                        self.exhausted = true;
                        return (0, 0);
                    }
                    left_idx -= 1;
                }
            }
        }
    }
    fn find_smallest_value_to_the_right_bigger_than_value_at_idx(
        &self,
        left: usize,
    ) -> Option<usize> {
        let mut smallest_value: Option<(char, usize)> = None;
        let mut right = left + 1;
        loop {
            if right >= self.chars.len() {
                return smallest_value.map(|tup| tup.1);
            }
            let right_ch = self.chars[right];
            if self.chars[left] < right_ch {
                match smallest_value {
                    None => smallest_value = Some((right_ch, right)),
                    Some(ref mut smallest) => if right_ch < smallest.0 {
                        smallest.0 = right_ch;
                        smallest.1 = right;
                    },
                }
            }
            right += 1;
        }
    }
}

#[test]
fn test_single_item() {
    let mut perms = permutations("f");
    macro_rules! assert_next_is {
        ($expected:expr) => {
            assert_eq!(perms.next().as_ref().map(|s| &**s), $expected);
        };
    }
    assert_next_is!(Some("f"));
    assert_next_is!(None);
}

#[test]
fn test_1234() {
    let mut perms = permutations("4213");
    macro_rules! assert_next_is {
        ($expected:expr) => {
            assert_eq!(perms.next().as_ref().map(|s| &**s), $expected);
        };
    }
    assert_next_is!(Some("1234"));
    assert_next_is!(Some("1243"));
    assert_next_is!(Some("1324"));
    assert_next_is!(Some("1342"));
    assert_next_is!(Some("1423"));
    assert_next_is!(Some("1432"));
    assert_next_is!(Some("2134"));
    assert_next_is!(Some("2143"));
    assert_next_is!(Some("2314"));
    assert_next_is!(Some("2341"));
    assert_next_is!(Some("2413"));
    assert_next_is!(Some("2431"));
    assert_next_is!(Some("3124"));
    assert_next_is!(Some("3142"));
    assert_next_is!(Some("3214"));
    assert_next_is!(Some("3241"));
    assert_next_is!(Some("3412"));
    assert_next_is!(Some("3421"));
    assert_next_is!(Some("4123"));
    assert_next_is!(Some("4132"));
    assert_next_is!(Some("4213"));
    assert_next_is!(Some("4231"));
    assert_next_is!(Some("4312"));
    assert_next_is!(Some("4321"));
    assert_next_is!(None);
}
