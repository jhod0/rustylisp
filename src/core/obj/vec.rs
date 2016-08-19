use std::iter::{IntoIterator, FromIterator};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct PersistentVec<T> {
    size: usize,
    capacity: usize,
    root: TrieNodeRef<T>,
}

type TrieNodeRef<T> = Rc<PersistentTrieNode<T>>;

#[derive(Clone, Debug)]
enum PersistentTrieNode<T> {
    Node(TrieNodeRef<T>,
         TrieNodeRef<T>),
    Leaf(Option<T>, Option<T>),
}

pub struct IntoIter<T> {
    size: usize,
    cur:  usize,
    cap:  usize,
    root: Rc<PersistentTrieNode<T>>,
}

pub struct Iter<'a, T: 'a> {
    size: usize,
    cur:  usize,
    cap:  usize,
    root: &'a PersistentTrieNode<T>,
}

impl<T: Clone> IntoIterator for PersistentVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            size: self.size,
            cur: 0,
            cap: self.capacity,
            root: self.root,
        }
    }
}

impl<T> FromIterator<T> for PersistentVec<T> {
    fn from_iter<I>(iter: I) -> Self
            where I: IntoIterator<Item=T> {
        // Not terribly efficient, but number of items must be known beforehand
        // in order to build the PersistentTrie
        let into_iter = iter.into_iter();
        let (mut the_iter, len) = match into_iter.size_hint() {
            (_, Some(high)) => {
                (Box::new(into_iter) as Box<Iterator<Item=T>>, high)
            },
            (_, None) => {
                let v: Vec<T> = into_iter.collect();
                let len       = v.len();
                let viter     = v.into_iter();

                (Box::new(viter) as Box<Iterator<Item=T>>, len)
            }
        };

        Self::from_iter_mut(&mut the_iter, len)
    }
}

impl<T: PartialEq> PartialEq for PersistentVec<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.size == other.size {
            for i in 0..other.size {
                if !self.lookup(i).eq(&other.lookup(i)) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}

impl<T> PersistentVec<T> {
    pub fn new() -> Self {
        PersistentVec {
            size: 0, capacity: 2,
            root: PersistentTrieNode::leaf(None, None).to_ref()
        }
    }

    pub fn concat<I>(iter: I) -> Self 
            where I: Iterator<Item=PersistentVec<T>>,
                  T: Clone {
        iter.flat_map(|v| v.into_iter())
            .collect()
    }

    pub fn with_size(size: usize) -> Self {
        let adjsize = {
            let adj = size.next_power_of_two();
            if adj < 2 {
                2
            } else {
                adj
            }
        };

        PersistentVec {
            size: size, capacity: adjsize,
            root: PersistentTrieNode::with_size(adjsize).to_ref()
        }
    }

    pub fn from_iter_mut<I>(iter: &mut I, size: usize) -> Self 
            where I: Iterator<Item=T> {
        let cap = {
            let c = size.next_power_of_two();
            if c < 2 {
                2
            } else {
                c
            }
        };
        PersistentVec {
            size: size, capacity: cap,
            root: PersistentTrieNode::from_iter_mut(iter, size, cap)
                                      .to_ref()
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        self.root.iter(self.size, self.capacity)
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn lookup(&self, index: usize) -> Option<&T> {
        if index >= self.size {
            None
        } else {
            let cap = if self.capacity >= 2 {
                self.capacity
            } else {
                2
            };
            self.root.lookup(index, cap)
        }
    }
}

impl<T: Clone> PersistentVec<T> {
    pub fn repeating(size: usize, item: T) -> Self {
        struct VecIter<T>(T);
        impl<T: Clone> Iterator for VecIter<T> {
            type Item=T;
            fn next(&mut self) -> Option<T> { Some(self.0.clone()) }
        }
        let mut iter = VecIter(item);
        Self::from_iter_mut(&mut iter, size)
    }

    pub fn insert(&self, index: usize, item: T) -> Option<Self> {
        if index >= self.size {
            None
        } else if let Some(new) = self.root.insert(index, self.capacity, item) {
            Some(PersistentVec {
                root: new.to_ref(),
                ..*self
            })
        } else {
            None
        }
    }

    pub fn push(&self, item: T) -> Self {
        if self.size < self.capacity {
            PersistentVec {
                size: self.size+1, 
                root: self.root.insert(self.size, self.capacity, item)
                               .unwrap().to_ref(),
                ..*self
            }
        } else {
            // TODO
            unimplemented!()
        }
    }
}

impl<T> PersistentTrieNode<T> {
    fn from_iter_mut<I>(source: &mut I, count: usize, cap: usize) -> Self 
            where I: Iterator<Item=T> {
        debug_assert!(cap.is_power_of_two());
        debug_assert!(count <= cap);
        match cap {
            0 => Self::leaf(None, None),
            1 => Self::leaf(source.next(), None),
            2 => Self::leaf(source.next(), source.next()),
            _ => {
                let next  = cap >> 1;
                let diff = if count >= next {
                    count - next
                } else {
                    0
                };
                let left  = Self::from_iter_mut(source, next, next);
                let right = Self::from_iter_mut(source, diff, next);
                Self::node(left.to_ref(), right.to_ref())
            }
        }
    }

    fn with_size(size: usize) -> Self {
        debug_assert!(size.is_power_of_two());

        if size == 2 {
            Self::leaf(None, None)
        } else { 
            let half = size >> 1;
            let l = Self::with_size(half);
            let r = Self::with_size(half);
            Self::node(l.to_ref(), r.to_ref())
        }
    }

    pub fn iter<'a>(&'a self, size: usize, cap: usize) -> Iter<'a, T> {
        Iter {
            size: size, cap: cap, cur: 0, root: self
        }
    }

    fn node(l: TrieNodeRef<T>, r: TrieNodeRef<T>) -> Self {
        PersistentTrieNode::Node(l, r)
    }

    fn leaf(l: Option<T>, r: Option<T>) -> Self {
        PersistentTrieNode::Leaf(l, r)
    }

    fn to_ref(self) -> TrieNodeRef<T> {
        Rc::new(self)
    }

    fn lookup(&self, index: usize, cap: usize) -> Option<&T> {
        debug_assert!(cap.is_power_of_two());
        let half = cap >> 1;

        let (left, new_index) = if index < half {
            (true, index)
        } else {
            (false, index % half)
        };

        match self {
            &PersistentTrieNode::Node(ref l, ref r) => {
                if left {
                    l.lookup(new_index, half)
                } else {
                    r.lookup(new_index, half)
                }
            },

            &PersistentTrieNode::Leaf(ref l, ref r) => {
                if cap != 2 {
                    panic!("Improper PersistentTrieNode")
                } else  {
                    match index {
                        0 => l.as_ref(),
                        1 => r.as_ref(),
                        _ => None,
                    }
                }
            }
        }
    }
}

impl<T: Clone> PersistentTrieNode<T> {
    fn insert(&self, index: usize, cap: usize, item: T) -> Option<Self> {
        debug_assert!(cap.is_power_of_two());
        let half = cap >> 1;

        let (left, new_index) = if index < half {
            (true, index)
        } else {
            (false, index % half)
        };

        match self {
            &PersistentTrieNode::Node(ref l, ref r) => {
                if left {
                    l.insert(new_index, half, item).map(|new_l| {
                        Self::node(new_l.to_ref(),
                                   r.clone())
                    })
                } else {
                    r.insert(new_index, half, item).map(|new_r| {
                        Self::node(l.clone(),
                                   new_r.to_ref())
                    })
                }
            },

            &PersistentTrieNode::Leaf(ref l, ref r) => {
                if cap != 2 {
                    panic!("Improper PersistentTrieNode")
                } else  {
                    match index {
                        0 => Some(Self::leaf(Some(item), r.clone())),
                        1 => Some(Self::leaf(l.clone(),  Some(item))),
                        _ => None,
                    }
                }
            }
        }
    }
}

impl<T: Clone> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.size {
            None
        } else {
            let next = self.root.lookup(self.cur, self.cap)
                                .expect("vec::Iter::next : invalid size");
            self.cur += 1;
            Some(next.clone())
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.size - self.cur;
        (n, Some(n))
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.size {
            None
        } else {
            let next = self.root.lookup(self.cur, self.cap)
                                .expect("vec::Iter::next : invalid size");
            self.cur += 1;
            Some(next)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.size - self.cur;
        (n, Some(n))
    }
}


#[cfg(test)]
mod test {
    use super::PersistentVec;

    #[test]
    fn test_insert_lookup() {
        let vec_1 = PersistentVec::with_size(10);
        for i in 0..10 {
            assert_eq!(vec_1.lookup(i), None)
        }

        let first = vec_1.insert(0, 0).unwrap();
        let vec_filled = (1..10).fold(first, |vec, n| {
            vec.insert(n, n).unwrap()
        });

        for i in 0..10 {
            assert_eq!(vec_1.lookup(i), None);
            assert_eq!(vec_filled.lookup(i), Some(&i));
        }
    }

    #[test]
    #[ignore]
    fn test_push() {
        let mut pvec = PersistentVec::new();

        for i in 0..10 {
            assert_eq!(pvec.len(), i);
            pvec = pvec.push(i);
            assert_eq!(i, *pvec.lookup(i).unwrap());
        }

        assert_eq!(pvec.len(), 10);
    }

    #[test]
    fn test_from_iter() {
        for len in 0..100 {
            let vec: Vec<u32> = (0..len).collect();
            let pvec          = vec.iter().map(|&n| n).collect::<PersistentVec<_>>();
            println!("vec:  {:?}", vec);
            println!("pvec: {:?}", pvec);

            assert!(vec.len() == pvec.len());
            for (&a, &b) in vec.iter().zip(pvec.iter()) {
                assert_eq!(a, b);
            }
        }
    }
}
