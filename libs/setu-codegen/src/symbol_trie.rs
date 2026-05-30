use setu_type_info::TypeInfo;
use std::collections::HashMap;

type Child = HashMap<Box<str>, Node>;

#[derive(Default, Debug)]
struct Node {
    count: usize,
    child: Child,
}

#[derive(Debug, Default)]
pub struct SymbolTrie {
    child: Child,
}

impl From<&TypeInfo> for SymbolTrie {
    fn from(info: &TypeInfo) -> Self {
        let mut trie = SymbolTrie::default();
        for path in info.registry.keys() {
            trie.insert(path);
        }
        trie
    }
}

impl SymbolTrie {
    pub fn insert(&mut self, path: &str) {
        let mut child = &mut self.child;

        for part in path.rsplit("::") {
            let node = child.entry(Box::from(part)).or_default();
            node.count += 1;
            child = &mut node.child;
        }
    }

    pub fn shortest_symbol<'a>(&self, path: &'a str) -> ShortestSymbolIter<'_, 'a> {
        ShortestSymbolIter {
            nodes: &self.child,
            parts: path.rsplit("::"),
            done: false,
        }
    }
}

pub struct ShortestSymbolIter<'this, 'path> {
    nodes: &'this Child,
    parts: std::str::RSplit<'path, &'static str>,
    done: bool,
}

impl<'a> Iterator for ShortestSymbolIter<'_, 'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let part = self.parts.next()?;
        let node = self.nodes.get(&Box::from(part))?;

        if node.count == 1 {
            self.done = true;
        }

        self.nodes = &node.child;
        Some(part)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortest_path() {
        let input = ["Z::A::X", "Z::A::B", "Z::X::A", "Z::X::B", "Y::X::A"];

        let mut trie = SymbolTrie::default();
        for s in input {
            trie.insert(s);
        }

        let c = |path| {
            let mut parts: Vec<_> = trie.shortest_symbol(path).collect();
            parts.reverse();
            parts.join("::")
        };

        assert_eq!(c("Z::A::X"), "X");
        assert_eq!(c("Z::A::B"), "A::B");
        assert_eq!(c("Z::X::A"), "Z::X::A");
        assert_eq!(c("Z::X::B"), "X::B");
        assert_eq!(c("Y::X::A"), "Y::X::A");
    }
}
