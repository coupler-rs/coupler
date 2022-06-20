use std::collections::HashMap;
use std::ops::Index;

pub type ParamId = u32;

pub trait Mapping {
    fn map(&self, value: f64) -> f64;
    fn unmap(&self, value: f64) -> f64;
}

struct DefaultMapping;

impl Mapping for DefaultMapping {
    fn map(&self, value: f64) -> f64 {
        value
    }

    fn unmap(&self, value: f64) -> f64 {
        value
    }
}

pub trait Format {
    fn parse(&self, string: &str) -> Result<f64, ()>;
    fn display(&self, value: f64, write: &mut dyn std::fmt::Write);
}

struct DefaultFormat;

impl Format for DefaultFormat {
    fn parse(&self, string: &str) -> Result<f64, ()> {
        string.parse().map_err(|_| ())
    }

    fn display(&self, value: f64, write: &mut dyn std::fmt::Write) {
        let _ = write!(write, "{}", value);
    }
}

pub trait Access<T> {
    fn get(&self, target: &T) -> f64;
    fn set(&self, target: &T, value: f64);
}

struct DefaultAccessor;

impl<T> Access<T> for DefaultAccessor {
    fn get(&self, _target: &T) -> f64 {
        0.0
    }

    fn set(&self, _target: &T, _value: f64) {}
}

pub struct ParamInfo<P> {
    id: ParamId,
    name: String,
    label: String,
    steps: Option<usize>,
    default: f64,
    mapping: Box<dyn Mapping + Send + Sync>,
    format: Box<dyn Format + Send + Sync>,
    accessor: Box<dyn Access<P> + Send + Sync>,
}

impl<P> ParamInfo<P> {
    #[inline]
    pub fn new(id: ParamId) -> Self {
        ParamInfo {
            id,
            name: String::new(),
            label: String::new(),
            steps: None,
            default: 0.0,
            mapping: Box::new(DefaultMapping),
            format: Box::new(DefaultFormat),
            accessor: Box::new(DefaultAccessor),
        }
    }

    #[inline]
    pub fn id(mut self, id: ParamId) -> Self {
        self.id = id;
        self
    }

    #[inline]
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    #[inline]
    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    #[inline]
    pub fn continuous(mut self) -> Self {
        self.steps = None;
        self
    }

    #[inline]
    pub fn discrete(mut self, steps: usize) -> Self {
        self.steps = Some(steps);
        self
    }

    #[inline]
    pub fn default(mut self, value: f64) -> Self {
        self.default = value;
        self
    }

    #[inline]
    pub fn mapping<M>(mut self, mapping: M) -> Self
    where
        M: Mapping + Send + Sync + 'static,
    {
        self.mapping = Box::new(mapping);
        self
    }

    #[inline]
    pub fn format<F>(mut self, format: F) -> Self
    where
        F: Format + Send + Sync + 'static,
    {
        self.format = Box::new(format);
        self
    }

    #[inline]
    pub fn accessor<A>(mut self, accessor: A) -> Self
    where
        A: Access<P> + Send + Sync + 'static,
    {
        self.accessor = Box::new(accessor);
        self
    }

    #[inline]
    pub fn get_id(&self) -> ParamId {
        self.id
    }

    #[inline]
    pub fn get_name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn get_label(&self) -> &str {
        &self.label
    }

    #[inline]
    pub fn get_steps(&self) -> Option<usize> {
        self.steps
    }

    #[inline]
    pub fn get_default(&self) -> f64 {
        self.default
    }

    #[inline]
    pub fn get_mapping(&self) -> &dyn Mapping {
        &*self.mapping
    }

    #[inline]
    pub fn get_format(&self) -> &dyn Format {
        &*self.format
    }

    #[inline]
    pub fn get_accessor(&self) -> &dyn Access<P> {
        &*self.accessor
    }
}

pub struct ParamList<P> {
    params: Vec<ParamInfo<P>>,
    index: HashMap<ParamId, usize>,
}

impl<P> ParamList<P> {
    #[inline]
    pub fn new() -> ParamList<P> {
        ParamList { params: Vec::new(), index: HashMap::new() }
    }

    #[inline]
    pub fn param(mut self, param: ParamInfo<P>) -> Self {
        assert!(self.index.get(&param.id).is_none(), "Duplicate parameter id {}", param.id);

        self.index.insert(param.id, self.params.len());
        self.params.push(param);
        self
    }

    #[inline]
    pub fn params(&self) -> &[ParamInfo<P>] {
        &self.params
    }

    #[inline]
    pub fn get(&self, id: ParamId) -> Option<&ParamInfo<P>> {
        self.index_of(id).map(|i| &self.params[i])
    }

    #[inline]
    pub fn index_of(&self, id: ParamId) -> Option<usize> {
        self.index.get(&id).copied()
    }
}

impl<P> Index<ParamId> for ParamList<P> {
    type Output = ParamInfo<P>;

    #[inline]
    fn index(&self, index: ParamId) -> &ParamInfo<P> {
        self.get(index).unwrap()
    }
}
