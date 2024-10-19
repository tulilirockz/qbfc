#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BrainfuckToken {
    Next,
    Prev,
    Add,
    Sub,
    Out,
    Input,
    LoopStart,
    LoopEnd,
    Invalid,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CompressedBrainfuckToken {
    pub token: BrainfuckToken,
    pub num: u64,
}

impl BrainfuckToken {
    pub fn to_opposite(self: Self) -> Option<BrainfuckToken> {
        match self {
            BrainfuckToken::Next => Some(Self::Prev),
            BrainfuckToken::Prev => Some(Self::Next),
            BrainfuckToken::Add => Some(Self::Sub),
            BrainfuckToken::Sub => Some(Self::Add),
            BrainfuckToken::LoopStart => Some(Self::LoopEnd),
            BrainfuckToken::LoopEnd => Some(Self::LoopStart),
            BrainfuckToken::Input => None,
            BrainfuckToken::Out => None,
            _ => None,
        }
    }
}

pub trait ValidTokenCollection {
    fn validate(self: &Self) -> bool;
}

impl ValidTokenCollection for Vec<CompressedBrainfuckToken> {
    fn validate(self: &Self) -> bool {
        let mut index: usize = 0;
        while index < self.len() {
            if self[index].token != BrainfuckToken::LoopStart {
                index += 1;
                continue;
            }

            let mut balance = 0;
            let mut subindex: usize = index;
            while subindex < self.len() {
                match self[subindex].token {
                    BrainfuckToken::LoopStart => balance += 1,
                    BrainfuckToken::LoopEnd => balance -= 1,
                    _ => (),
                }

                if balance == 0 {
                    break;
                }

                subindex += 1;
            }

            if balance != 0 {
                return false;
            }

            index += 1;
        }
        true
    }
}

pub trait CompressableTokenCollection {
    fn compress(self: Self) -> Vec<CompressedBrainfuckToken>;
}

pub trait BfToken {
    fn is_valid_token(self: Self) -> bool;
}

pub trait CleanableTokenCollection {
    fn clean(self: Self) -> Self;
}

impl CleanableTokenCollection for Vec<CompressedBrainfuckToken> {
    fn clean(self: Self) -> Self {
        let mut cleaned = self;
        let mut idx: usize = 0;
        while idx < cleaned.len() - 1 {
            if !cleaned[idx + 1].token.to_opposite().is_none()
                && cleaned[idx].token == cleaned[idx + 1].token.to_opposite().unwrap()
            {
                if cleaned[idx].num - cleaned[idx + 1].num == 0 {
                    cleaned.remove(idx);
                    cleaned.remove(idx);
                } else if cleaned[idx].num - cleaned[idx + 1].num > 0 {
                    cleaned[idx].num = cleaned[idx].num - cleaned[idx + 1].num;
                    cleaned.remove(idx + 1);
                } else if cleaned[idx + 1].num - cleaned[idx].num > 0 {
                    cleaned[idx + 1].num = cleaned[idx + 1].num - cleaned[idx].num;
                    cleaned.remove(idx);
                }
                continue;
            }
            idx += 1;
        }
        cleaned
    }
}

impl CompressableTokenCollection for Vec<BrainfuckToken> {
    fn compress(self: Self) -> Vec<CompressedBrainfuckToken> {
        let mut compressed_tokens: Vec<CompressedBrainfuckToken> = Vec::new();
        let mut index: usize = 0;
        while index < self.len() {
            let currtoken = &self[index];
            match self[index] {
                BrainfuckToken::Out
                | BrainfuckToken::Input
                | BrainfuckToken::LoopStart
                | BrainfuckToken::LoopEnd => {
                    compressed_tokens.push(CompressedBrainfuckToken {
                        token: currtoken.to_owned(),
                        num: 1,
                    });
                    index += 1;
                    continue;
                }
                _ => (),
            }

            let mut numtokens: u64 = 0;
            let mut subindex: usize = index;
            while currtoken == &self[subindex] {
                numtokens += 1;
                subindex += 1;

                if subindex == self.len() {
                    break;
                }
            }

            compressed_tokens.push(CompressedBrainfuckToken {
                token: currtoken.to_owned(),
                num: numtokens,
            });

            index += subindex - index;
        }
        return compressed_tokens;
    }
}

impl Into<BrainfuckToken> for u8 {
    fn into(self) -> BrainfuckToken {
        match self {
            b'>' => BrainfuckToken::Next,
            b'<' => BrainfuckToken::Prev,
            b'+' => BrainfuckToken::Add,
            b'-' => BrainfuckToken::Sub,
            b'.' => BrainfuckToken::Out,
            b',' => BrainfuckToken::Input,
            b'[' => BrainfuckToken::LoopStart,
            b']' => BrainfuckToken::LoopEnd,
            _ => BrainfuckToken::Invalid,
        }
    }
}

impl BfToken for u8 {
    fn is_valid_token(self: Self) -> bool {
        match self {
            b'>' | b'<' | b'+' | b'-' | b'.' | b',' | b'[' | b']' => true,
            _ => false,
        }
    }
}
