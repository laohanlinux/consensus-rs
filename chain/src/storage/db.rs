// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;
use std::collections::btree_map::{BTreeMap, IntoIter as BtmIntoIter, Iter as BtmIter, Range};
use std::collections::hash_map::{Entry as HmEntry, IntoIter as HmIntoIter, Iter as HmIter};
use std::collections::Bound::*;
use std::cmp::Ordering::*;
use std::iter::{Iterator as StdIterator, Peekable};

/// Map containing changes with corresponding key.
#[derive(Debug, Clone)]
pub struct Changes{
    data: BTreeMap<Vec<u8>, Change>,
}

impl Changes {
    /// Create a new empty `Changes` instance.
    fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    /// Returns iterator over changes.
    pub fn iter(&self) -> BtmIter<Vec<u8>, Change> {
        self.data.iter()
    }
}

/// Iterator over the `Changes` data.
#[derive(Debug)]
pub struct ChangesIterator{
    inner: BtmIntoIter<Vec<u8>, Change>,
}

impl StdIterator for ChangesIterator {
    type Item = (Vec<u8>, Change);

    fn next(&mut self) -> Option<Self::Item> {self.inner.next()}
}

impl IntoIterator for Changes {
    type Item = (Vec<u8>, Change);
    type IntoIter = ChangesIterator;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter{
            inner: self.data.into_iter(),
        }
    }
}


/// A set of serial changes that should be applied to a storage atomically.
#[derive(Debug, Clone)]
pub struct Patch {
    changes: HashMap<String, Changes>,
}

impl Patch {
    /// Craetes a new empty `Patch` instance.
    fn new() -> Self {
        Self {
            changes: HashMap::new(),
        }
    }

    /// Returns changes for the given name.
    fn changes(&self, name: &str) -> Option<&Changes>{
        self.changes.get(name)
    }

    /// Returns a mutable reference to the changes corresponding to the `name`.
    fn changes_mut(&mut self, name: &str) -> Option<&mut Changes> {
        self.changes.get_mut(name)
    }

    /// Gets the corresponding entry in the map by the given name for in-place manipulation.
    fn changes_entry(&mut self, name: String) -> HmEntry<String, Changes> {
        self.changes.entry(name)
    }

    /// Inserts changes with the given name.
    fn insert_changes(&mut self, name: String, changes: Changes) {
        self.changes.insert(name, changes);
    }

    /// Returns iterator over changes
    pub fn iter(&self) -> HmIter<String, Changes> {
        self.changes.iter()
    }

    /// Returns the number of changes.
    pub fn len(&self) -> usize {
        self.changes.iter().fold(0, |acc, (_, changes)| acc
        + changes.data.len())
    }

    /// Returns `true` if this patch contains no changes and `false` otherwise.
    pub fn is_empty(&self) -> bool {self.len() == 0}
}

/// Iterator over the `Patch` data.
#[derive(Debug)]
pub struct PatchIterator{
    inner: HmIntoIter<String, Changes>,
}

impl StdIterator for PatchIterator{
    type Item = (String, Changes);

    fn next(&mut self) -> Option<Self::Item> {self.inner.next()}
}

impl IntoIterator for Patch {
    type Item = (String, Changes);
    type IntoIter = PatchIterator;

    fn into_iter(self) -> Self::IntoIter{
        Self::IntoIter{
            inner: self.changes.into_iter(),
        }
    }
}

/// A generalized iterator over the storage views.
pub type Iter<'a> = Box<Iterator + 'a>;

/// An enum that represents a kind of change to some key in the storage.
#[derive(Debug, Clone, PartialOrd)]
pub enum Change {
    /// Put the specified value into the storage for the corresponding key.
    Put(Vec<u8>),
    /// Delete a value from the storage for the corresponding key.
    Delete,
}

// FIXME: make &mut Fork "unwind safe" (ECR-176)
pub struct Fork {
    snapshot: Box<Snapshot>,
    patch: Patch,
    changelog: Vec<(String, Vec<u8>, Option<Change>)>,
    logged: bool,
}

pub struct ForkIter <'a> {
    snapshot: Iter<'a>,
    changes: Option<Peekable<Range<'a, Vec<u8>, Changes>>>,
}

#[derive(Debug, PartialEq, Eq)]
enum NextIterValue {
    Stored,
    Replaced,
    Inserted,
    Deleted,
    MissDeleted,
    Finished,
}

pub trait Database: Send + Sync + 'static {
    /// Creates a new snapshot of the database from its current state.
    fn snapshot(&self) -> Box<Snapshot>;

    /// Creates a new fork of the database from its current state.
    fn fork(&self) -> Fork {
        Fork{
            snapshot: self.snapshot(),
            patch: Patch::new(),
            changelog: Vec::new(),
            logged: false,
        }
    }

    fn merge(&self, patch: Patch) -> Result<()>;

    fn merge_sync(&self, patch: Patch) -> Result<()>;
}

pub trait Snapshot: 'static {
    fn get(&self, name: &str, key: &[u8]) -> Option<Vec<u8>>;

    fn contains(&self, name: &str, key: &[u8]) -> bool {
        self.get(name, key).is_some()
    }

    fn iter<'a>(&'a self, name: &'str, from: &[u8]) -> Iter<'a>;
}

pub trait Iterator{
    fn next(&mut self) -> Option<(&[u8], &[u8])>;

    /// Returns references to the current key and value of the iterator.
    fn peek(&mut self) -> Option<(&[u8], &[u8]>;
}

impl Snapshot for Fork {
    fn get(&self, name: &str, key: &[u8]) -> Option<Vec<u8>> {
        if let Some(changes) = self.patch.chanegs(name) {
            if let Some(change) = changes.data.get(key) {
                match *change {
                    Change::Put(ref v) => return Some(v.clone()),
                    Change::Delete => return None,
                }
            }
        }
        self.snapshot.get(name, key)
    }

    fn contains(&self, name: &str, key: &[u8]) -> bool {
        if let Some(changes) = self.patch.chanegs(name) {
            match *changes {
                Change::Put(..) => return true,
                Change::Delete => return false,
            }
        }
        self.snapshot.contains(name, key)
    }

    fn iter<'a>(&'a self, name: &str, from: &[u8]) -> Iter<'a> {
        let range = (Included(from), Unbounded);
        let changes = match self.patch.changes(name) {
            Some(changes) => Some(changes.data.range::<[u8], _>(range).peekable()),
            None => None,
        };

        Box::new(ForkIter {
            snapshot: self.snapshot.iter(name, from),
            changes,
        })
    }
}

impl Fork {
    pub fn checkpoint(&mut self) {
        if self.logged {
            panic!("call checkpoint before rollback or commit");
        }
        self.logged = true;
    }

    pub fn commit(&mut self) {
        if !self.logged {
            panic!("call commit before checkpoint");
        }
        self.changelog.clear();
        self.logged = false;
    }

    pub fn rollback(&mut self) {
        if !self.logged {
            panic!("call rollback before checkpoint");
        }
        // reverse
        for (name, k , c) in self.changelog.drain(..).rev() {
            // find the target patch
            if let Some(changes) = self.patch.changes_mut(&name) {
                match c {
                    Some(change) => changes.data.insert(k, change),
                    None => changes.data.remove(&k),
                };
            }
        }
        self.logged = false;
    }

    pub fn put(&mut self, name: &str, key: Vec<u8>, value: Vec<u8>) {
        let changes = self.patch
            .changes_entry(name.to_string())
            .or_insert_with(Changes::new);
        if self.logged {
            self.changelog.push((
                name.to_string(),
                key.clone(),
                changes.data.insert(key, Change::Put(value)),
            ));
        }else {
            changes.data.insert(key, Change::Put(value));
        }
    }

    pub fn remove(&mut self, name: &str, key: Vec<u8>) {
        let changes: = self.patch
            .changes_entry(name.to_string())
            .or_insert_with(Changes::new);
        if self.logged{
            self.changeslog.push((
                name.to_string(),
                key.clone(),
                changes.data.insert(key, Change::Delete),
            ));
        }else {
            changes.data.insert(key, Change::Delete);
        }
    }


    /// Removes all keys starting with the specified prefix from the column familly
    /// with the given `name`.
    pub fn remove_by_prefix(&mut self, name: &str, prefix: Option<&Vec<u8>>) {
        let changes = self.patch
            .changes_entry(name.to_string())
            .or_insert_with(Changes::new());
        // Remove changes
        if let Some(prefix) = prefix {
            let keys = changes
                .data.range::<Vec<u8>, _>((Included(prefix), Unbounded))
                .map(|(k, _)| k.to_vec())
                .take_while(|k| k.start_with(prefix))
                .collect::<Vec<_>>();
            for k in keys{
                changes.data.remove(&k);
            }
        }else {
            changes.data.clear();
        }

        // Remove from storage
        let mut iter = self.snapshot
            .iter(name, prefix.map_or(&[], |k| k.as_slice()));
        while let Some((k, ..)) = iter.next() {
            let change = changes.data.insert(k.to_vec(), Change::Delete);
            if self.logged {
                self.changelog.push((name.to_string(), k.to_vec(), change));
            }
        }
    }
	
}
