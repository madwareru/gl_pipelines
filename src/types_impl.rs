#[derive(Clone, Copy, Debug)]
pub enum UniformType {
    /// One 32-bit wide float (equivalent to `f32`)
    Float1,
    /// Two 32-bit wide floats (equivalent to `[f32; 2]`)
    Float2,
    /// Three 32-bit wide floats (equivalent to `[f32; 3]`)
    Float3,
    /// Four 32-bit wide floats (equivalent to `[f32; 4]`)
    Float4,
    /// One unsigned 32-bit integers (equivalent to `[u32; 1]`)
    Int1,
    /// Two unsigned 32-bit integers (equivalent to `[u32; 2]`)
    Int2,
    /// Three unsigned 32-bit integers (equivalent to `[u32; 3]`)
    Int3,
    /// Four unsigned 32-bit integers (equivalent to `[u32; 4]`)
    Int4,
    /// Four by four matrix of 32-bit floats
    Mat4,
}

impl UniformType {
    /// Byte size for a given UniformType
    pub fn size(&self) -> usize {
        match self {
            UniformType::Float1 => 4,
            UniformType::Float2 => 8,
            UniformType::Float3 => 12,
            UniformType::Float4 => 16,
            UniformType::Int1 => 4,
            UniformType::Int2 => 8,
            UniformType::Int3 => 12,
            UniformType::Int4 => 16,
            UniformType::Mat4 => 64,
        }
    }
}

pub struct UniformDesc {
    pub(crate) name: String,
    pub(crate) uniform_type: UniformType,
    pub(crate) array_count: usize,
}

pub struct UniformBlockLayout {
    pub uniforms: Vec<UniformDesc>,
}

impl UniformDesc {
    pub fn new(name: &str, uniform_type: UniformType) -> UniformDesc {
        UniformDesc {
            name: name.to_string(),
            uniform_type,
            array_count: 1,
        }
    }

    pub fn array(self, array_count: usize) -> UniformDesc {
        UniformDesc {
            array_count,
            ..self
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum VertexFormat {
    /// One 32-bit wide float (equivalent to `f32`)
    Float1,
    /// Two 32-bit wide floats (equivalent to `[f32; 2]`)
    Float2,
    /// Three 32-bit wide floats (equivalent to `[f32; 3]`)
    Float3,
    /// Four 32-bit wide floats (equivalent to `[f32; 4]`)
    Float4,
    /// One unsigned 8-bit integer (equivalent to `u8`)
    Byte1,
    /// Two unsigned 8-bit integers (equivalent to `[u8; 2]`)
    Byte2,
    /// Three unsigned 8-bit integers (equivalent to `[u8; 3]`)
    Byte3,
    /// Four unsigned 8-bit integers (equivalent to `[u8; 4]`)
    Byte4,
    /// One unsigned 16-bit integer (equivalent to `u16`)
    Short1,
    /// Two unsigned 16-bit integers (equivalent to `[u16; 2]`)
    Short2,
    /// Tree unsigned 16-bit integers (equivalent to `[u16; 3]`)
    Short3,
    /// Four unsigned 16-bit integers (equivalent to `[u16; 4]`)
    Short4,
    /// One unsigned 32-bit integers (equivalent to `[u32; 1]`)
    Int1,
    /// Two unsigned 32-bit integers (equivalent to `[u32; 2]`)
    Int2,
    /// Three unsigned 32-bit integers (equivalent to `[u32; 3]`)
    Int3,
    /// Four unsigned 32-bit integers (equivalent to `[u32; 4]`)
    Int4,
    /// Four by four matrix of 32-bit floats
    Mat4,
}

impl VertexFormat {
    pub fn size(&self) -> i32 {
        match self {
            VertexFormat::Float1 => 1,
            VertexFormat::Float2 => 2,
            VertexFormat::Float3 => 3,
            VertexFormat::Float4 => 4,
            VertexFormat::Byte1 => 1,
            VertexFormat::Byte2 => 2,
            VertexFormat::Byte3 => 3,
            VertexFormat::Byte4 => 4,
            VertexFormat::Short1 => 1,
            VertexFormat::Short2 => 2,
            VertexFormat::Short3 => 3,
            VertexFormat::Short4 => 4,
            VertexFormat::Int1 => 1,
            VertexFormat::Int2 => 2,
            VertexFormat::Int3 => 3,
            VertexFormat::Int4 => 4,
            VertexFormat::Mat4 => 16,
        }
    }

    pub fn byte_len(&self) -> i32 {
        match self {
            VertexFormat::Float1 => 1 * 4,
            VertexFormat::Float2 => 2 * 4,
            VertexFormat::Float3 => 3 * 4,
            VertexFormat::Float4 => 4 * 4,
            VertexFormat::Byte1 => 1,
            VertexFormat::Byte2 => 2,
            VertexFormat::Byte3 => 3,
            VertexFormat::Byte4 => 4,
            VertexFormat::Short1 => 1 * 2,
            VertexFormat::Short2 => 2 * 2,
            VertexFormat::Short3 => 3 * 2,
            VertexFormat::Short4 => 4 * 2,
            VertexFormat::Int1 => 1 * 4,
            VertexFormat::Int2 => 2 * 4,
            VertexFormat::Int3 => 3 * 4,
            VertexFormat::Int4 => 4 * 4,
            VertexFormat::Mat4 => 16 * 4,
        }
    }

    pub(crate) fn type_(&self) -> u32 {
        match self {
            VertexFormat::Float1 => glow::FLOAT,
            VertexFormat::Float2 => glow::FLOAT,
            VertexFormat::Float3 => glow::FLOAT,
            VertexFormat::Float4 => glow::FLOAT,
            VertexFormat::Byte1 => glow::UNSIGNED_BYTE,
            VertexFormat::Byte2 => glow::UNSIGNED_BYTE,
            VertexFormat::Byte3 => glow::UNSIGNED_BYTE,
            VertexFormat::Byte4 => glow::UNSIGNED_BYTE,
            VertexFormat::Short1 => glow::UNSIGNED_SHORT,
            VertexFormat::Short2 => glow::UNSIGNED_SHORT,
            VertexFormat::Short3 => glow::UNSIGNED_SHORT,
            VertexFormat::Short4 => glow::UNSIGNED_SHORT,
            VertexFormat::Int1 => glow::UNSIGNED_INT,
            VertexFormat::Int2 => glow::UNSIGNED_INT,
            VertexFormat::Int3 => glow::UNSIGNED_INT,
            VertexFormat::Int4 => glow::UNSIGNED_INT,
            VertexFormat::Mat4 => glow::FLOAT,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum VertexStep {
    PerVertex,
    PerInstance,
}

impl Default for VertexStep {
    fn default() -> VertexStep {
        VertexStep::PerVertex
    }
}

#[derive(Clone, Debug)]
pub struct BufferLayout {
    pub stride: i32,
    pub step_func: VertexStep,
    pub step_rate: i32,
}

impl Default for BufferLayout {
    fn default() -> BufferLayout {
        BufferLayout {
            stride: 0,
            step_func: VertexStep::PerVertex,
            step_rate: 1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct VertexAttribute {
    pub name: &'static str,
    pub format: VertexFormat,
    pub buffer_index: usize,
}

impl VertexAttribute {
    pub const fn new(name: &'static str, format: VertexFormat) -> VertexAttribute {
        Self::with_buffer(name, format, 0)
    }

    pub const fn with_buffer(
        name: &'static str,
        format: VertexFormat,
        buffer_index: usize,
    ) -> VertexAttribute {
        VertexAttribute {
            name,
            format,
            buffer_index,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PipelineLayout {
    pub buffers: &'static [BufferLayout],
    pub attributes: &'static [VertexAttribute],
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BlendState {
    pub(crate) equation: Equation,
    pub(crate) sfactor: BlendFactor,
    pub(crate) dfactor: BlendFactor,
}

impl BlendState {
    pub fn new(equation: Equation, sfactor: BlendFactor, dfactor: BlendFactor) -> BlendState {
        BlendState {
            equation,
            sfactor,
            dfactor,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StencilState {
    pub front: StencilFaceState,
    pub back: StencilFaceState,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StencilFaceState {
    /// Operation to use when stencil test fails
    pub fail_op: StencilOp,

    /// Operation to use when stencil test passes, but depth test fails
    pub depth_fail_op: StencilOp,

    /// Operation to use when both stencil and depth test pass,
    /// or when stencil pass and no depth or depth disabled
    pub pass_op: StencilOp,

    /// Used for stencil testing with test_ref and test_mask: if (test_ref & test_mask) *test_func* (*stencil* && test_mask)
    /// Default is Always, which means "always pass"
    pub test_func: CompareFunc,

    /// Default value: 0
    pub test_ref: i32,

    /// Default value: all 1s
    pub test_mask: u32,

    /// Specifies a bit mask to enable or disable writing of individual bits in the stencil planes
    /// Default value: all 1s
    pub write_mask: u32,
}

/// Operations performed on current stencil value when comparison test passes or fails.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StencilOp {
    /// Default value
    Keep,
    Zero,
    Replace,
    IncrementClamp,
    DecrementClamp,
    Invert,
    IncrementWrap,
    DecrementWrap,
}

/// Depth and stencil compare function
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CompareFunc {
    /// Default value
    Always,
    Never,
    Less,
    Equal,
    LessOrEqual,
    Greater,
    NotEqual,
    GreaterOrEqual,
}

/// Specifies how incoming RGBA values (source) and the RGBA in framebuffer (destination)
/// are combined.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Equation {
    /// Adds source and destination. Source and destination are multiplied
    /// by blending parameters before addition.
    Add,
    /// Subtracts destination from source. Source and destination are
    /// multiplied by blending parameters before subtraction.
    Subtract,
    /// Subtracts source from destination. Source and destination are
    /// multiplied by blending parameters before subtraction.
    ReverseSubtract,
}

/// Blend values.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BlendValue {
    SourceColor,
    SourceAlpha,
    DestinationColor,
    DestinationAlpha,
}

/// Blend factors.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BlendFactor {
    Zero,
    One,
    Value(BlendValue),
    OneMinusValue(BlendValue),
    SourceAlphaSaturate,
}

impl Default for Equation {
    fn default() -> Equation {
        Equation::Add
    }
}

impl From<Equation> for u32 {
    fn from(eq: Equation) -> Self {
        match eq {
            Equation::Add => glow::FUNC_ADD,
            Equation::Subtract => glow::FUNC_SUBTRACT,
            Equation::ReverseSubtract => glow::FUNC_REVERSE_SUBTRACT,
        }
    }
}

impl From<BlendFactor> for u32 {
    fn from(factor: BlendFactor) -> u32 {
        match factor {
            BlendFactor::Zero => glow::ZERO,
            BlendFactor::One => glow::ONE,
            BlendFactor::Value(BlendValue::SourceColor) => glow::SRC_COLOR,
            BlendFactor::Value(BlendValue::SourceAlpha) => glow::SRC_ALPHA,
            BlendFactor::Value(BlendValue::DestinationColor) => glow::DST_COLOR,
            BlendFactor::Value(BlendValue::DestinationAlpha) => glow::DST_ALPHA,
            BlendFactor::OneMinusValue(BlendValue::SourceColor) => glow::ONE_MINUS_SRC_COLOR,
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha) => glow::ONE_MINUS_SRC_ALPHA,
            BlendFactor::OneMinusValue(BlendValue::DestinationColor) => glow::ONE_MINUS_DST_COLOR,
            BlendFactor::OneMinusValue(BlendValue::DestinationAlpha) => glow::ONE_MINUS_DST_ALPHA,
            BlendFactor::SourceAlphaSaturate => glow::SRC_ALPHA_SATURATE,
        }
    }
}

impl From<StencilOp> for u32 {
    fn from(op: StencilOp) -> Self {
        match op {
            StencilOp::Keep => glow::KEEP,
            StencilOp::Zero => glow::ZERO,
            StencilOp::Replace => glow::REPLACE,
            StencilOp::IncrementClamp => glow::INCR,
            StencilOp::DecrementClamp => glow::DECR,
            StencilOp::Invert => glow::INVERT,
            StencilOp::IncrementWrap => glow::INCR_WRAP,
            StencilOp::DecrementWrap => glow::DECR_WRAP,
        }
    }
}

impl From<CompareFunc> for u32 {
    fn from(cf: CompareFunc) -> Self {
        match cf {
            CompareFunc::Always => glow::ALWAYS,
            CompareFunc::Never => glow::NEVER,
            CompareFunc::Less => glow::LESS,
            CompareFunc::Equal => glow::EQUAL,
            CompareFunc::LessOrEqual => glow::LEQUAL,
            CompareFunc::Greater => glow::GREATER,
            CompareFunc::NotEqual => glow::NOTEQUAL,
            CompareFunc::GreaterOrEqual => glow::GEQUAL,
        }
    }
}

/// Specify whether front- or back-facing polygons can be culled.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CullFace {
    Nothing,
    Front,
    Back,
}

/// Define front- and back-facing polygons.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FrontFaceOrder {
    Clockwise,
    CounterClockwise,
}

/// A pixel-wise comparison function.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Comparison {
    Never,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Equal,
    NotEqual,
    Always,
}

impl From<Comparison> for u32 {
    fn from(cmp: Comparison) -> Self {
        match cmp {
            Comparison::Never => glow::NEVER,
            Comparison::Less => glow::LESS,
            Comparison::LessOrEqual => glow::LEQUAL,
            Comparison::Greater => glow::GREATER,
            Comparison::GreaterOrEqual => glow::GEQUAL,
            Comparison::Equal => glow::EQUAL,
            Comparison::NotEqual => glow::NOTEQUAL,
            Comparison::Always => glow::ALWAYS,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PrimitiveType {
    Triangles,
    Lines,
}

impl From<PrimitiveType> for u32 {
    fn from(primitive_type: PrimitiveType) -> Self {
        match primitive_type {
            PrimitiveType::Triangles => glow::TRIANGLES,
            PrimitiveType::Lines => glow::LINES,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IndexType {
    Byte,
    Short,
    Int,
}

impl From<IndexType> for u32 {
    fn from(index_type: IndexType) -> Self {
        match index_type {
            IndexType::Byte => glow::UNSIGNED_BYTE,
            IndexType::Short => glow::UNSIGNED_SHORT,
            IndexType::Int => glow::UNSIGNED_INT,
        }
    }
}

impl IndexType {
    pub fn for_type<T>() -> IndexType {
        match std::mem::size_of::<T>() {
            1 => IndexType::Byte,
            2 => IndexType::Short,
            4 => IndexType::Int,
            _ => panic!("Unsupported index buffer index type"),
        }
    }

    pub fn size(self) -> u8 {
        match self {
            IndexType::Byte => 1,
            IndexType::Short => 2,
            IndexType::Int => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BufferType {
    VertexBuffer,
    IndexBuffer,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Usage {
    Immutable,
    Dynamic,
    Stream,
}