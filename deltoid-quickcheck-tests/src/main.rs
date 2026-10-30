use deltoid::{Apply, Core, Delta, RwLock, Str};
use deltoid_derive::Delta;
use quickcheck::{quickcheck, Arbitrary, Gen};
use serde_derive::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Delta)]
#[derive(Serialize, Deserialize)]
pub struct Basic {
    pub field: Option<u32>,
}

impl Arbitrary for Basic {
    fn arbitrary(g: &mut Gen) -> Self {
        Basic { field: Arbitrary::arbitrary(g) }
    }
}

fn tester(a: Basic, b: Basic) -> bool {
    // Roundtrip property: a.apply(a.delta(&b)) == b

    println!("a: {:?}", a);
    println!("b: {:?}", b);
    let delta = a.delta(&b).unwrap();
    println!("delta: {:?}", delta);
    let c = a.apply(delta).unwrap();
    println!("c: {:?}", c);
    println!("b == c: {:?}", b == c);
    println!();
    return b == c;
}

/// Shared roundtrip property for the composed test cases:
/// 1. `a.apply(a.delta(&b)) == b`
/// 2. the same must hold when the delta takes a detour through JSON,
///    since every `Core::Delta` is required to be serde-de/serializable.
///
/// Errors from `delta`/`apply`/serde are unexpected for total roundtrips
/// and are surfaced as panics (quickcheck catches them and reports the
/// witness), while a value mismatch returns `false` so quickcheck shrinks.
fn roundtrip<T>(a: T, b: T) -> bool
where T: Apply + Delta
{
    let delta = a.delta(&b).unwrap();
    let c = a.apply(delta.clone()).unwrap();
    if b != c { return false }
    let json = serde_json::to_string(&delta).unwrap();
    let delta2: <T as Core>::Delta = serde_json::from_str(&json).unwrap();
    let c2 = a.apply(delta2).unwrap();
    b == c2
}

fn property<T: Arbitrary + std::fmt::Debug + 'static>(name: &str, f: fn(T, T) -> bool) {
    println!("== quickcheck: {} ==", name);
    quickcheck(f);
}

// ---------------------------------------------------------------------------
// Case 1: nested struct + data enum + Vec<String>
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub enum Color {
    Red,
    Rgb(u8, u8, u8),
    Named(String),
}

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub struct Pixel {
    pub pos: Point,
    pub color: Color,
    pub tags: Vec<String>,
    pub id: u64,
}

impl Arbitrary for Point {
    fn arbitrary(g: &mut Gen) -> Self {
        Point { x: Arbitrary::arbitrary(g), y: Arbitrary::arbitrary(g) }
    }
}

impl Arbitrary for Color {
    fn arbitrary(g: &mut Gen) -> Self {
        match u8::arbitrary(g) % 3 {
            0 => Color::Red,
            1 => Color::Rgb(
                Arbitrary::arbitrary(g),
                Arbitrary::arbitrary(g),
                Arbitrary::arbitrary(g),
            ),
            _ => Color::Named(Arbitrary::arbitrary(g)),
        }
    }
}

impl Arbitrary for Pixel {
    fn arbitrary(g: &mut Gen) -> Self {
        Pixel {
            pos: Arbitrary::arbitrary(g),
            color: Arbitrary::arbitrary(g),
            tags: (0..usize::arbitrary(g) % 6)
                .map(|_| String::arbitrary(g))
                .collect(),
            id: Arbitrary::arbitrary(g),
        }
    }
}

// ---------------------------------------------------------------------------
// Case 2: BTreeMap<String, struct> + enum with struct variant + 3-tuple
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub enabled: bool,
    pub level: u8,
}

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub enum Mode {
    Auto,
    Manual { speed: i16, label: String },
}

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub struct Settings {
    pub configs: BTreeMap<String, Config>,
    pub mode: Mode,
    pub retries: Option<u32>,
    pub version: (u16, u16, char),
}

impl Arbitrary for Config {
    fn arbitrary(g: &mut Gen) -> Self {
        Config {
            name: Arbitrary::arbitrary(g),
            enabled: Arbitrary::arbitrary(g),
            level: Arbitrary::arbitrary(g),
        }
    }
}

impl Arbitrary for Mode {
    fn arbitrary(g: &mut Gen) -> Self {
        match u8::arbitrary(g) % 2 {
            0 => Mode::Auto,
            _ => Mode::Manual {
                speed: Arbitrary::arbitrary(g),
                label: Arbitrary::arbitrary(g),
            },
        }
    }
}

impl Arbitrary for Settings {
    fn arbitrary(g: &mut Gen) -> Self {
        Settings {
            configs: Arbitrary::arbitrary(g),
            mode: Arbitrary::arbitrary(g),
            retries: Arbitrary::arbitrary(g),
            version: Arbitrary::arbitrary(g),
        }
    }
}

// ---------------------------------------------------------------------------
// Case 3: Vec of Box-carrying enum + BTreeSet + Option<Box<struct>>
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub struct Leaf {
    pub value: i64,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub enum Op {
    Insert(Box<Leaf>),
    Delete { label: String },
    Flush,
}

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub struct OpsLog {
    pub ops: Vec<Op>,
    pub seen: BTreeSet<u32>,
    pub last: Option<Box<Leaf>>,
    pub size: usize,
}

impl Arbitrary for Leaf {
    fn arbitrary(g: &mut Gen) -> Self {
        Leaf { value: Arbitrary::arbitrary(g), label: Arbitrary::arbitrary(g) }
    }
}

impl Arbitrary for Op {
    fn arbitrary(g: &mut Gen) -> Self {
        match u8::arbitrary(g) % 3 {
            0 => Op::Insert(Box::new(Arbitrary::arbitrary(g))),
            1 => Op::Delete { label: Arbitrary::arbitrary(g) },
            _ => Op::Flush,
        }
    }
}

impl Arbitrary for OpsLog {
    fn arbitrary(g: &mut Gen) -> Self {
        OpsLog {
            ops: (0..usize::arbitrary(g) % 8).map(|_| Op::arbitrary(g)).collect(),
            seen: Arbitrary::arbitrary(g),
            last: Arbitrary::arbitrary(g),
            size: Arbitrary::arbitrary(g),
        }
    }
}

// ---------------------------------------------------------------------------
// Case 4: VecDeque + nested array + tuple with enum + Option<String>
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub struct Event {
    pub time: u64,
    pub kind: String,
}

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub enum Severity {
    Low,
    High(u8),
}

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub struct Log {
    pub events: VecDeque<Event>,
    pub matrix: [[u16; 3]; 2],
    pub alarm: (Severity, bool),
    pub note: Option<String>,
}

impl Arbitrary for Event {
    fn arbitrary(g: &mut Gen) -> Self {
        Event { time: Arbitrary::arbitrary(g), kind: Arbitrary::arbitrary(g) }
    }
}

impl Arbitrary for Severity {
    fn arbitrary(g: &mut Gen) -> Self {
        match u8::arbitrary(g) % 2 {
            0 => Severity::Low,
            _ => Severity::High(Arbitrary::arbitrary(g)),
        }
    }
}

impl Arbitrary for Log {
    fn arbitrary(g: &mut Gen) -> Self {
        Log {
            events: Arbitrary::arbitrary(g),
            matrix: Arbitrary::arbitrary(g),
            alarm: (Arbitrary::arbitrary(g), Arbitrary::arbitrary(g)),
            note: Arbitrary::arbitrary(g),
        }
    }
}

// ---------------------------------------------------------------------------
// Case 5: Result<enum, String> + Range<u32> + HashMap
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub enum Outcome {
    Success(u32),
    Failure { code: i32, msg: String },
}

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub struct Sensor {
    pub id: u16,
    pub unit: String,
}

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub struct Measurement {
    pub sensor: Sensor,
    pub outcome: Result<Outcome, String>,
    pub span: Range<u32>,
    pub lookup: HashMap<String, u32>,
}

impl Arbitrary for Outcome {
    fn arbitrary(g: &mut Gen) -> Self {
        match u8::arbitrary(g) % 2 {
            0 => Outcome::Success(Arbitrary::arbitrary(g)),
            _ => Outcome::Failure {
                code: Arbitrary::arbitrary(g),
                msg: Arbitrary::arbitrary(g),
            },
        }
    }
}

impl Arbitrary for Sensor {
    fn arbitrary(g: &mut Gen) -> Self {
        Sensor { id: Arbitrary::arbitrary(g), unit: Arbitrary::arbitrary(g) }
    }
}

impl Arbitrary for Measurement {
    fn arbitrary(g: &mut Gen) -> Self {
        Measurement {
            sensor: Arbitrary::arbitrary(g),
            outcome: if bool::arbitrary(g) {
                Ok(Arbitrary::arbitrary(g))
            } else {
                Err(Arbitrary::arbitrary(g))
            },
            span: Arbitrary::arbitrary(g),
            lookup: Arbitrary::arbitrary(g),
        }
    }
}

// ---------------------------------------------------------------------------
// Case 6: recursive tree (Vec of Self) + BTreeMap<String, enum> + BTreeSet
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub struct TreeNode {
    pub value: u32,
    pub name: String,
    pub children: Vec<TreeNode>,
}

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub enum Marker {
    Flag,
    Note(String),
}

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub struct Forest {
    pub roots: Vec<TreeNode>,
    pub markers: BTreeMap<String, Marker>,
    pub tags: BTreeSet<String>,
    pub depth: u8,
}

impl TreeNode {
    /// Depth-capped generation so trees stay bounded regardless of `g.size()`.
    fn arbitrary_depth(g: &mut Gen, depth: u32) -> Self {
        let children = if depth == 0 {
            Vec::new()
        } else {
            (0..usize::arbitrary(g) % 3)
                .map(|_| Self::arbitrary_depth(g, depth - 1))
                .collect()
        };
        TreeNode {
            value: Arbitrary::arbitrary(g),
            name: Arbitrary::arbitrary(g),
            children,
        }
    }
}

impl Arbitrary for TreeNode {
    fn arbitrary(g: &mut Gen) -> Self {
        Self::arbitrary_depth(g, 3)
    }
}

impl Arbitrary for Marker {
    fn arbitrary(g: &mut Gen) -> Self {
        match u8::arbitrary(g) % 2 {
            0 => Marker::Flag,
            _ => Marker::Note(Arbitrary::arbitrary(g)),
        }
    }
}

impl Arbitrary for Forest {
    fn arbitrary(g: &mut Gen) -> Self {
        Forest {
            roots: (0..usize::arbitrary(g) % 4)
                .map(|_| TreeNode::arbitrary(g))
                .collect(),
            markers: Arbitrary::arbitrary(g),
            tags: Arbitrary::arbitrary(g),
            depth: Arbitrary::arbitrary(g),
        }
    }
}

// ---------------------------------------------------------------------------
// Case 7: Str<'static> + Cow<'static, String> + string-heavy composites
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub enum Text {
    Plain(String),
    Rich { bold: bool, body: Cow<'static, String> },
}

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub alias: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub struct Article {
    pub title: Str<'static>,
    pub author: Author,
    pub body: Text,
    pub comments: Vec<String>,
    pub revisions: BTreeMap<u32, String>,
}

impl Arbitrary for Text {
    fn arbitrary(g: &mut Gen) -> Self {
        match u8::arbitrary(g) % 2 {
            0 => Text::Plain(Arbitrary::arbitrary(g)),
            _ => Text::Rich {
                bold: Arbitrary::arbitrary(g),
                body: Cow::Owned(Arbitrary::arbitrary(g)),
            },
        }
    }
}

impl Arbitrary for Author {
    fn arbitrary(g: &mut Gen) -> Self {
        Author { name: Arbitrary::arbitrary(g), alias: Arbitrary::arbitrary(g) }
    }
}

impl Arbitrary for Article {
    fn arbitrary(g: &mut Gen) -> Self {
        Article {
            title: Str::from(String::arbitrary(g)),
            author: Arbitrary::arbitrary(g),
            body: Arbitrary::arbitrary(g),
            comments: (0..usize::arbitrary(g) % 6)
                .map(|_| String::arbitrary(g))
                .collect(),
            revisions: Arbitrary::arbitrary(g),
        }
    }
}

// ---------------------------------------------------------------------------
// Case 8: Arc/Rc shared composites + RwLock + Option<Box<enum>>
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub struct Counter {
    pub total: u64,
    pub flag: bool,
}

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub enum Command {
    Ping,
    Set(Rc<Counter>),
    Reset { after: u32 },
}

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub struct State {
    pub current: Arc<Counter>,
    pub pending: Option<Box<Command>>,
    pub history: Vec<Rc<Counter>>,
    pub limit: RwLock<u32>,
}

impl Arbitrary for Counter {
    fn arbitrary(g: &mut Gen) -> Self {
        Counter { total: Arbitrary::arbitrary(g), flag: Arbitrary::arbitrary(g) }
    }
}

impl Arbitrary for Command {
    fn arbitrary(g: &mut Gen) -> Self {
        match u8::arbitrary(g) % 3 {
            0 => Command::Ping,
            1 => Command::Set(Rc::new(Arbitrary::arbitrary(g))),
            _ => Command::Reset { after: Arbitrary::arbitrary(g) },
        }
    }
}

impl Arbitrary for State {
    fn arbitrary(g: &mut Gen) -> Self {
        State {
            current: Arc::new(Arbitrary::arbitrary(g)),
            pending: Arbitrary::arbitrary(g),
            history: (0..usize::arbitrary(g) % 4)
                .map(|_| Rc::new(Counter::arbitrary(g)))
                .collect(),
            limit: RwLock::new(Arbitrary::arbitrary(g)),
        }
    }
}

// ---------------------------------------------------------------------------
// Case 9: recursive expression enum + BTreeMap<String, Expr> + Vec<struct>
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub enum Expr {
    Lit(i64),
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
}

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub struct Var {
    pub name: String,
    pub index: u32,
}

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub struct Program {
    pub vars: Vec<Var>,
    pub funcs: BTreeMap<String, Expr>,
    pub entry: Expr,
    pub debug: bool,
}

impl Expr {
    /// Depth-capped generation; `Add` branches twice per level.
    fn arbitrary_depth(g: &mut Gen, depth: u32) -> Self {
        if depth == 0 {
            return Expr::Lit(Arbitrary::arbitrary(g));
        }
        match u8::arbitrary(g) % 3 {
            0 => Expr::Lit(Arbitrary::arbitrary(g)),
            1 => Expr::Neg(Box::new(Self::arbitrary_depth(g, depth - 1))),
            _ => Expr::Add(
                Box::new(Self::arbitrary_depth(g, depth - 1)),
                Box::new(Self::arbitrary_depth(g, depth - 1)),
            ),
        }
    }
}

impl Arbitrary for Expr {
    fn arbitrary(g: &mut Gen) -> Self {
        Self::arbitrary_depth(g, 3)
    }
}

impl Arbitrary for Var {
    fn arbitrary(g: &mut Gen) -> Self {
        Var { name: Arbitrary::arbitrary(g), index: Arbitrary::arbitrary(g) }
    }
}

impl Arbitrary for Program {
    fn arbitrary(g: &mut Gen) -> Self {
        Program {
            vars: (0..usize::arbitrary(g) % 6).map(|_| Var::arbitrary(g)).collect(),
            funcs: (0..usize::arbitrary(g) % 6)
                .map(|_| (String::arbitrary(g), Expr::arbitrary(g)))
                .collect(),
            entry: Arbitrary::arbitrary(g),
            debug: Arbitrary::arbitrary(g),
        }
    }
}

// ---------------------------------------------------------------------------
// Case 10: HashSet<struct> + Vec<(struct, enum)> + [Option<struct>; 3] + i128
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Delta, Serialize, Deserialize)]
pub struct Inner {
    pub flag: bool,
    pub count: u8,
}

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub enum Middle {
    A(Inner),
    B { items: Vec<Inner>, tag: String },
    C,
}

#[derive(Clone, Debug, PartialEq, Delta, Serialize, Deserialize)]
pub struct Outer {
    pub uniq: HashSet<Inner>,
    pub pairs: Vec<(Inner, Middle)>,
    pub maybe: Option<Box<Middle>>,
    pub grid: [Option<Inner>; 3],
    pub counter: i128,
}

impl Arbitrary for Inner {
    fn arbitrary(g: &mut Gen) -> Self {
        Inner { flag: Arbitrary::arbitrary(g), count: Arbitrary::arbitrary(g) }
    }
}

impl Arbitrary for Middle {
    fn arbitrary(g: &mut Gen) -> Self {
        match u8::arbitrary(g) % 3 {
            0 => Middle::A(Arbitrary::arbitrary(g)),
            1 => Middle::B {
                items: (0..usize::arbitrary(g) % 4)
                    .map(|_| Inner::arbitrary(g))
                    .collect(),
                tag: Arbitrary::arbitrary(g),
            },
            _ => Middle::C,
        }
    }
}

impl Arbitrary for Outer {
    fn arbitrary(g: &mut Gen) -> Self {
        Outer {
            uniq: (0..usize::arbitrary(g) % 8)
                .map(|_| Inner::arbitrary(g))
                .collect::<HashSet<_>>(),
            pairs: (0..usize::arbitrary(g) % 6)
                .map(|_| (Inner::arbitrary(g), Middle::arbitrary(g)))
                .collect(),
            maybe: Arbitrary::arbitrary(g),
            grid: Arbitrary::arbitrary(g),
            counter: Arbitrary::arbitrary(g),
        }
    }
}

fn main() {
    quickcheck(tester as fn(Basic, Basic) -> bool);

    property::<Pixel>("Pixel: Point + Color + Vec<String> + u64", roundtrip::<Pixel>);
    property::<Settings>("Settings: BTreeMap<String, Config> + Mode + Option<u32> + (u16, u16, char)", roundtrip::<Settings>);
    property::<OpsLog>("OpsLog: Vec<Op> + BTreeSet<u32> + Option<Box<Leaf>> + usize", roundtrip::<OpsLog>);
    property::<Log>("Log: VecDeque<Event> + [[u16; 3]; 2] + (Severity, bool) + Option<String>", roundtrip::<Log>);
    property::<Measurement>("Measurement: Sensor + Result<Outcome, String> + Range<u32> + HashMap<String, u32>", roundtrip::<Measurement>);
    property::<Forest>("Forest: Vec<TreeNode> (recursive) + BTreeMap<String, Marker> + BTreeSet<String> + u8", roundtrip::<Forest>);
    property::<Article>("Article: Str<'static> + Author + Text(Cow<'static, String>) + Vec<String> + BTreeMap<u32, String>", roundtrip::<Article>);
    property::<State>("State: Arc<Counter> + Option<Box<Command(Rc<Counter>)>> + Vec<Rc<Counter>> + RwLock<u32>", roundtrip::<State>);
    property::<Program>("Program: Vec<Var> + BTreeMap<String, Expr> (recursive) + Expr + bool", roundtrip::<Program>);
    property::<Outer>("Outer: HashSet<Inner> + Vec<(Inner, Middle)> + Option<Box<Middle>> + [Option<Inner>; 3] + i128", roundtrip::<Outer>);
}
