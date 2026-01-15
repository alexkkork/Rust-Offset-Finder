// Tue Jan 15 2026 - Alex

use crate::luau::opcode::LuauOpcode;
use crate::luau::bytecode::{LuauBytecode, BytecodeInstruction, BytecodeConstant};
use crate::memory::MemoryReader;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Luau bytecode decompiler
pub struct LuauDecompiler {
    reader: Arc<dyn MemoryReader>,
    indent_size: usize,
    emit_comments: bool,
}

impl LuauDecompiler {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self {
            reader,
            indent_size: 2,
            emit_comments: true,
        }
    }

    pub fn with_indent(mut self, size: usize) -> Self {
        self.indent_size = size;
        self
    }

    pub fn without_comments(mut self) -> Self {
        self.emit_comments = false;
        self
    }

    /// Decompile bytecode to Lua source
    pub fn decompile(&self, bytecode: &LuauBytecode) -> DecompilationResult {
        let mut result = DecompilationResult::new();
        let mut ctx = DecompilerContext::new(bytecode);

        // Analyze control flow
        ctx.analyze_control_flow();

        // Generate output
        result.source = self.generate_source(&ctx);
        result.warnings = ctx.warnings.clone();
        result.success = true;

        result
    }

    fn generate_source(&self, ctx: &DecompilerContext) -> String {
        let mut output = String::new();

        if self.emit_comments {
            output.push_str("-- Decompiled Luau bytecode\n\n");
        }

        // Process instructions
        for (i, insn) in ctx.bytecode.instructions().iter().enumerate() {
            let stmt = self.instruction_to_statement(ctx, insn, i);
            if !stmt.is_empty() {
                output.push_str(&stmt);
                output.push('\n');
            }
        }

        output
    }

    fn instruction_to_statement(&self, ctx: &DecompilerContext, insn: &BytecodeInstruction, _pc: usize) -> String {
        match insn.opcode {
            LuauOpcode::LoadNil => {
                let reg = self.reg_name(ctx, insn.a);
                format!("{} = nil", reg)
            }
            LuauOpcode::LoadB => {
                let reg = self.reg_name(ctx, insn.a);
                let value = if insn.b != 0 { "true" } else { "false" };
                format!("{} = {}", reg, value)
            }
            LuauOpcode::LoadN => {
                let reg = self.reg_name(ctx, insn.a);
                format!("{} = {}", reg, insn.d as f64)
            }
            LuauOpcode::LoadK => {
                let reg = self.reg_name(ctx, insn.a);
                let constant = self.get_constant(ctx, insn.d as usize);
                format!("{} = {}", reg, constant)
            }
            LuauOpcode::Move => {
                let dst = self.reg_name(ctx, insn.a);
                let src = self.reg_name(ctx, insn.b);
                format!("{} = {}", dst, src)
            }
            LuauOpcode::GetGlobal => {
                let reg = self.reg_name(ctx, insn.a);
                let name = insn.aux.map(|a| self.get_constant(ctx, a as usize))
                    .unwrap_or_else(|| "global".to_string());
                format!("{} = {}", reg, name)
            }
            LuauOpcode::SetGlobal => {
                let name = insn.aux.map(|a| self.get_constant(ctx, a as usize))
                    .unwrap_or_else(|| "global".to_string());
                let reg = self.reg_name(ctx, insn.a);
                format!("{} = {}", name, reg)
            }
            LuauOpcode::GetUpval => {
                let reg = self.reg_name(ctx, insn.a);
                format!("{} = upvalue[{}]", reg, insn.b)
            }
            LuauOpcode::SetUpval => {
                let reg = self.reg_name(ctx, insn.a);
                format!("upvalue[{}] = {}", insn.b, reg)
            }
            LuauOpcode::GetTable => {
                let dst = self.reg_name(ctx, insn.a);
                let table = self.reg_name(ctx, insn.b);
                let key = self.reg_name(ctx, insn.c);
                format!("{} = {}[{}]", dst, table, key)
            }
            LuauOpcode::SetTable => {
                let table = self.reg_name(ctx, insn.b);
                let key = self.reg_name(ctx, insn.c);
                let value = self.reg_name(ctx, insn.a);
                format!("{}[{}] = {}", table, key, value)
            }
            LuauOpcode::NewTable => {
                let reg = self.reg_name(ctx, insn.a);
                format!("{} = {{}}", reg)
            }
            LuauOpcode::NewClosure => {
                let reg = self.reg_name(ctx, insn.a);
                format!("{} = function() end", reg)
            }
            LuauOpcode::Add => {
                self.binary_op(ctx, insn, "+")
            }
            LuauOpcode::Sub => {
                self.binary_op(ctx, insn, "-")
            }
            LuauOpcode::Mul => {
                self.binary_op(ctx, insn, "*")
            }
            LuauOpcode::Div => {
                self.binary_op(ctx, insn, "/")
            }
            LuauOpcode::Mod => {
                self.binary_op(ctx, insn, "%")
            }
            LuauOpcode::Pow => {
                self.binary_op(ctx, insn, "^")
            }
            LuauOpcode::Concat => {
                self.binary_op(ctx, insn, "..")
            }
            LuauOpcode::Not => {
                let dst = self.reg_name(ctx, insn.a);
                let src = self.reg_name(ctx, insn.b);
                format!("{} = not {}", dst, src)
            }
            LuauOpcode::Minus => {
                let dst = self.reg_name(ctx, insn.a);
                let src = self.reg_name(ctx, insn.b);
                format!("{} = -{}", dst, src)
            }
            LuauOpcode::Length => {
                let dst = self.reg_name(ctx, insn.a);
                let src = self.reg_name(ctx, insn.b);
                format!("{} = #{}", dst, src)
            }
            LuauOpcode::Call => {
                let func = self.reg_name(ctx, insn.a);
                let nargs = insn.b;
                let nrets = insn.c;
                
                let args: Vec<String> = (1..nargs as u8)
                    .map(|i| self.reg_name(ctx, insn.a + i))
                    .collect();
                
                if nrets == 0 {
                    format!("{}({})", func, args.join(", "))
                } else {
                    let rets: Vec<String> = (0..nrets as u8)
                        .map(|i| self.reg_name(ctx, insn.a + i))
                        .collect();
                    format!("{} = {}({})", rets.join(", "), func, args.join(", "))
                }
            }
            LuauOpcode::Return => {
                if insn.b == 1 {
                    "return".to_string()
                } else {
                    let values: Vec<String> = (0..(insn.b - 1) as u8)
                        .map(|i| self.reg_name(ctx, insn.a + i))
                        .collect();
                    format!("return {}", values.join(", "))
                }
            }
            LuauOpcode::Jump => {
                format!("-- jump {}", insn.d)
            }
            LuauOpcode::JumpBack => {
                format!("-- jumpback {}", insn.d)
            }
            LuauOpcode::JumpIf => {
                let cond = self.reg_name(ctx, insn.a);
                format!("if {} then -- jump {}", cond, insn.d)
            }
            LuauOpcode::JumpIfNot => {
                let cond = self.reg_name(ctx, insn.a);
                format!("if not {} then -- jump {}", cond, insn.d)
            }
            LuauOpcode::JumpIfEq => {
                let lhs = self.reg_name(ctx, insn.a);
                let rhs = insn.aux.map(|a| self.reg_name(ctx, a as u8))
                    .unwrap_or_else(|| "?".to_string());
                format!("if {} == {} then -- jump", lhs, rhs)
            }
            LuauOpcode::JumpIfNotEq => {
                let lhs = self.reg_name(ctx, insn.a);
                let rhs = insn.aux.map(|a| self.reg_name(ctx, a as u8))
                    .unwrap_or_else(|| "?".to_string());
                format!("if {} ~= {} then -- jump", lhs, rhs)
            }
            LuauOpcode::JumpIfLt => {
                let lhs = self.reg_name(ctx, insn.a);
                let rhs = insn.aux.map(|a| self.reg_name(ctx, a as u8))
                    .unwrap_or_else(|| "?".to_string());
                format!("if {} < {} then -- jump", lhs, rhs)
            }
            LuauOpcode::JumpIfLe => {
                let lhs = self.reg_name(ctx, insn.a);
                let rhs = insn.aux.map(|a| self.reg_name(ctx, a as u8))
                    .unwrap_or_else(|| "?".to_string());
                format!("if {} <= {} then -- jump", lhs, rhs)
            }
            LuauOpcode::ForNPrep => {
                format!("-- for numeric prep")
            }
            LuauOpcode::ForNLoop => {
                format!("-- for numeric loop")
            }
            LuauOpcode::ForGPrep => {
                format!("-- for generic prep")
            }
            LuauOpcode::ForGLoop => {
                format!("-- for generic loop")
            }
            _ => {
                if self.emit_comments {
                    format!("-- {:?}", insn.opcode)
                } else {
                    String::new()
                }
            }
        }
    }

    fn binary_op(&self, ctx: &DecompilerContext, insn: &BytecodeInstruction, op: &str) -> String {
        let dst = self.reg_name(ctx, insn.a);
        let lhs = self.reg_name(ctx, insn.b);
        let rhs = self.reg_name(ctx, insn.c);
        format!("{} = {} {} {}", dst, lhs, op, rhs)
    }

    fn reg_name(&self, ctx: &DecompilerContext, reg: u8) -> String {
        if let Some(name) = ctx.local_names.get(&(reg as usize)) {
            name.clone()
        } else {
            format!("r{}", reg)
        }
    }

    fn get_constant(&self, ctx: &DecompilerContext, index: usize) -> String {
        if let Some(constant) = ctx.bytecode.get_constant(index) {
            match constant {
                BytecodeConstant::Nil => "nil".to_string(),
                BytecodeConstant::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
                BytecodeConstant::Number(n) => format!("{}", n),
                BytecodeConstant::String(s) => format!("\"{}\"", s),
                BytecodeConstant::Table(_) => "{}".to_string(),
                BytecodeConstant::Closure(idx) => format!("function_{}", idx),
                BytecodeConstant::Import(i) => format!("import_{}", i),
            }
        } else {
            format!("K{}", index)
        }
    }
}

/// Context for decompilation
struct DecompilerContext<'a> {
    bytecode: &'a LuauBytecode,
    local_names: HashMap<usize, String>,
    warnings: Vec<String>,
    block_starts: HashSet<usize>,
}

impl<'a> DecompilerContext<'a> {
    fn new(bytecode: &'a LuauBytecode) -> Self {
        Self {
            bytecode,
            local_names: HashMap::new(),
            warnings: Vec::new(),
            block_starts: HashSet::new(),
        }
    }

    fn analyze_control_flow(&mut self) {
        for (pc, insn) in self.bytecode.instructions().iter().enumerate() {
            match insn.opcode {
                LuauOpcode::Jump | LuauOpcode::JumpBack |
                LuauOpcode::JumpIf | LuauOpcode::JumpIfNot |
                LuauOpcode::JumpIfEq | LuauOpcode::JumpIfNotEq |
                LuauOpcode::JumpIfLt | LuauOpcode::JumpIfLe => {
                    let target = (pc as i32 + insn.d as i32 + 1) as usize;
                    self.block_starts.insert(target);
                }
                _ => {}
            }
        }
    }
}

/// Result of decompilation
#[derive(Debug, Clone)]
pub struct DecompilationResult {
    pub source: String,
    pub warnings: Vec<String>,
    pub success: bool,
}

impl DecompilationResult {
    pub fn new() -> Self {
        Self {
            source: String::new(),
            warnings: Vec::new(),
            success: false,
        }
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

impl Default for DecompilationResult {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DecompilationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source)?;
        if !self.warnings.is_empty() {
            writeln!(f, "\n-- Warnings:")?;
            for warn in &self.warnings {
                writeln!(f, "-- {}", warn)?;
            }
        }
        Ok(())
    }
}

/// Bytecode analyzer for deeper analysis
pub struct BytecodeAnalyzer {
    reader: Arc<dyn MemoryReader>,
}

impl BytecodeAnalyzer {
    pub fn new(reader: Arc<dyn MemoryReader>) -> Self {
        Self { reader }
    }

    /// Analyze bytecode for patterns
    pub fn analyze(&self, bytecode: &LuauBytecode) -> BytecodeAnalysis {
        let mut analysis = BytecodeAnalysis::new();

        // Count opcode frequency
        for insn in bytecode.instructions() {
            *analysis.opcode_frequency.entry(insn.opcode).or_default() += 1;
        }

        // Find function calls
        for (pc, insn) in bytecode.instructions().iter().enumerate() {
            if matches!(insn.opcode, LuauOpcode::Call | LuauOpcode::GetGlobal) {
                analysis.function_calls.push(pc);
            }
        }

        // Calculate metrics
        analysis.instruction_count = bytecode.instruction_count();
        analysis.constant_count = bytecode.constant_count();
        analysis.complexity = self.calculate_complexity(bytecode);

        analysis
    }

    fn calculate_complexity(&self, bytecode: &LuauBytecode) -> f64 {
        let mut complexity = 1.0;

        for insn in bytecode.instructions() {
            match insn.opcode {
                LuauOpcode::Jump | LuauOpcode::JumpBack |
                LuauOpcode::JumpIf | LuauOpcode::JumpIfNot => {
                    complexity += 1.0;
                }
                LuauOpcode::ForNPrep | LuauOpcode::ForGPrep |
                LuauOpcode::ForNLoop | LuauOpcode::ForGLoop => {
                    complexity += 2.0;
                }
                LuauOpcode::Call => {
                    complexity += 0.5;
                }
                LuauOpcode::NewClosure => {
                    complexity += 3.0;
                }
                _ => {}
            }
        }

        complexity
    }
}

/// Analysis results
#[derive(Debug, Clone)]
pub struct BytecodeAnalysis {
    pub opcode_frequency: HashMap<LuauOpcode, usize>,
    pub function_calls: Vec<usize>,
    pub instruction_count: usize,
    pub constant_count: usize,
    pub complexity: f64,
}

impl BytecodeAnalysis {
    pub fn new() -> Self {
        Self {
            opcode_frequency: HashMap::new(),
            function_calls: Vec::new(),
            instruction_count: 0,
            constant_count: 0,
            complexity: 0.0,
        }
    }

    pub fn most_common_opcodes(&self, n: usize) -> Vec<(LuauOpcode, usize)> {
        let mut sorted: Vec<_> = self.opcode_frequency.iter().map(|(k, v)| (*k, *v)).collect();
        sorted.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        sorted.truncate(n);
        sorted
    }
}

impl Default for BytecodeAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

/// Constant value representation for decompiler output
#[derive(Debug, Clone)]
pub enum Constant {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Table,
    Closure(usize),
}

/// Constant propagation analyzer
pub struct ConstantPropagation {
    values: HashMap<usize, PropagatedValue>,
}

impl ConstantPropagation {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn analyze(&mut self, bytecode: &LuauBytecode) {
        for insn in bytecode.instructions() {
            match insn.opcode {
                LuauOpcode::LoadNil => {
                    self.values.insert(insn.a as usize, PropagatedValue::Nil);
                }
                LuauOpcode::LoadB => {
                    self.values.insert(insn.a as usize, PropagatedValue::Boolean(insn.b != 0));
                }
                LuauOpcode::LoadN => {
                    self.values.insert(insn.a as usize, PropagatedValue::Number(insn.d as f64));
                }
                LuauOpcode::LoadK => {
                    self.values.insert(insn.a as usize, PropagatedValue::Constant(insn.d as usize));
                }
                LuauOpcode::Move => {
                    if let Some(value) = self.values.get(&(insn.b as usize)).cloned() {
                        self.values.insert(insn.a as usize, value);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn get_value(&self, reg: usize) -> Option<&PropagatedValue> {
        self.values.get(&reg)
    }

    pub fn is_constant(&self, reg: usize) -> bool {
        self.values.contains_key(&reg)
    }
}

impl Default for ConstantPropagation {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum PropagatedValue {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Constant(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompilation_result() {
        let mut result = DecompilationResult::new();
        result.source = "local x = 1".to_string();
        result.success = true;
        
        assert!(result.success);
        assert!(!result.has_warnings());
    }

    #[test]
    fn test_bytecode_analysis() {
        let analysis = BytecodeAnalysis::new();
        assert_eq!(analysis.instruction_count, 0);
        assert_eq!(analysis.complexity, 0.0);
    }
}
