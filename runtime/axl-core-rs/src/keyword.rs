use crate::ir::*;
use crate::compact::CompactParseError;

pub struct KeywordParser {
    source: String,
    lines: Vec<String>,
    position: usize,
}

impl KeywordParser {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            lines: source.lines().map(String::from).collect(),
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Program, CompactParseError> {
        let mut instructions = Vec::new();
        
        while self.position < self.lines.len() {
            let line = self.lines[self.position].trim().to_string();
            self.position += 1;
            
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some(inst) = self.parse_line(&line)? {
                instructions.push(inst);
            }
        }
        
        Ok(Program { instructions })
    }
    
    fn parse_line(&mut self, line: &str) -> Result<Option<Instruction>, CompactParseError> {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        let keyword = parts[0];
        let rest = parts.get(1).unwrap_or(&"");
        
        match keyword {
            "let" => self.parse_let(rest),
            "emit" => self.parse_emit(rest),
            "return" => self.parse_return(rest),
            "if" => self.parse_if(rest),
            "while" => self.parse_while(rest),
            "fn" => self.parse_function(rest),
            "agent" => self.parse_agent(rest),
            "workflow" => self.parse_workflow(rest),
            "run" => self.parse_run(rest),
            "memory" => self.parse_memory_write(rest),
            "forget" => self.parse_forget(rest),
            _ => {
                // Try as expression
                let expr = self.parse_expression(line)?;
                Ok(Some(Instruction::Emit(expr)))
            }
        }
    }
    
    fn parse_let(&mut self, rest: &str) -> Result<Option<Instruction>, CompactParseError> {
        let parts: Vec<&str> = rest.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(CompactParseError("let requires 'name = expression'".into()));
        }
        let name = parts[0].trim().to_string();
        let expr = self.parse_expression(parts[1].trim())?;
        Ok(Some(Instruction::Let {
            target: name,
            value: expr,
            type_name: None,
        }))
    }
    
    fn parse_emit(&mut self, rest: &str) -> Result<Option<Instruction>, CompactParseError> {
        let expr = self.parse_expression(rest)?;
        Ok(Some(Instruction::Emit(expr)))
    }
    
    fn parse_return(&mut self, rest: &str) -> Result<Option<Instruction>, CompactParseError> {
        let expr = self.parse_expression(rest)?;
        Ok(Some(Instruction::Return(expr)))
    }
    
    fn parse_if(&mut self, rest: &str) -> Result<Option<Instruction>, CompactParseError> {
        let condition = self.parse_expression(rest)?;
        let mut body = Vec::new();
        let mut else_body = Vec::new();
        let mut in_else = false;
        
        while self.position < self.lines.len() {
            let line = self.lines[self.position].trim().to_string();
            self.position += 1;
            
            if line == "end" {
                break;
            }
            if line == "else" {
                in_else = true;
                continue;
            }
            
            if let Some(inst) = self.parse_line(&line)? {
                if in_else {
                    else_body.push(inst);
                } else {
                    body.push(inst);
                }
            }
        }
        
        Ok(Some(Instruction::If(If { condition, body, else_body })))
    }
    
    fn parse_while(&mut self, rest: &str) -> Result<Option<Instruction>, CompactParseError> {
        let condition = self.parse_expression(rest)?;
        let mut body = Vec::new();
        
        while self.position < self.lines.len() {
            let line = self.lines[self.position].trim().to_string();
            self.position += 1;
            
            if line == "end" {
                break;
            }
            
            if let Some(inst) = self.parse_line(&line)? {
                body.push(inst);
            }
        }
        
        Ok(Some(Instruction::While(While { condition, body })))
    }
    
    fn parse_function(&mut self, rest: &str) -> Result<Option<Instruction>, CompactParseError> {
        // fn name(params) -> type
        let parts: Vec<&str> = rest.splitn(2, '(').collect();
        let name = parts[0].trim().to_string();
        let params_str = parts.get(1).unwrap_or(&"");
        
        let (params_str, return_type) = if let Some(idx) = params_str.find("->") {
            (params_str[..idx].trim(), params_str[idx+2..].trim().to_string())
        } else {
            (params_str.trim(), "int".to_string())
        };
        
        let mut params = Vec::new();
        if !params_str.is_empty() {
            for param in params_str.split(',') {
                let (pname, ptype) = param.split_once(':').unwrap_or((param.trim(), "int"));
                params.push(Parameter {
                    name: pname.trim().to_string(),
                    type_name: ptype.trim().to_string(),
                });
            }
        }
        
        let mut body = Vec::new();
        while self.position < self.lines.len() {
            let line = self.lines[self.position].trim().to_string();
            self.position += 1;
            
            if line == "end" {
                break;
            }
            
            if let Some(inst) = self.parse_line(&line)? {
                body.push(inst);
            }
        }
        
        Ok(Some(Instruction::Function(Function {
            name, parameters: params, return_type, body,
        })))
    }
    
    fn parse_agent(&mut self, rest: &str) -> Result<Option<Instruction>, CompactParseError> {
        let name = rest.trim().to_string();
        let mut body = Vec::new();
        
        while self.position < self.lines.len() {
            let line = self.lines[self.position].trim().to_string();
            self.position += 1;
            
            if line == "end" {
                break;
            }
            
            if let Some(inst) = self.parse_line(&line)? {
                body.push(inst);
            }
        }
        
        Ok(Some(Instruction::Agent(Agent {
            name,
            tools: vec![],
            body,
            goal: None,
            tool_defs: vec![],
            memory_defs: vec![],
            handlers: vec![],
        })))
    }
    
    fn parse_workflow(&mut self, rest: &str) -> Result<Option<Instruction>, CompactParseError> {
        let name = rest.trim().to_string();
        let mut body = Vec::new();
        
        while self.position < self.lines.len() {
            let line = self.lines[self.position].trim().to_string();
            self.position += 1;
            
            if line == "end" {
                break;
            }
            
            if let Some(inst) = self.parse_line(&line)? {
                body.push(inst);
            }
        }
        
        Ok(Some(Instruction::Workflow(Workflow {
            name, body, handlers: vec![],
        })))
    }
    
    fn parse_run(&mut self, rest: &str) -> Result<Option<Instruction>, CompactParseError> {
        let name = rest.trim().to_string();
        Ok(Some(Instruction::Run(name)))
    }
    
    fn parse_memory_write(&mut self, rest: &str) -> Result<Option<Instruction>, CompactParseError> {
        let parts: Vec<&str> = rest.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(CompactParseError("memory requires 'key = value'".into()));
        }
        let key = parts[0].trim().to_string();
        let value = self.parse_expression(parts[1].trim())?;
        Ok(Some(Instruction::MemoryWrite(MemoryWrite {
            key,
            value,
            confidence: 100,
            ttl_seconds: None,
            source: "program".into(),
            tags: vec![],
        })))
    }
    
    fn parse_forget(&mut self, rest: &str) -> Result<Option<Instruction>, CompactParseError> {
        let key = rest.trim().to_string();
        Ok(Some(Instruction::Forget(key)))
    }
    
    fn parse_expression(&self, expr: &str) -> Result<Expression, CompactParseError> {
        let expr = expr.trim();
        
        // String literal
        if expr.starts_with('"') && expr.ends_with('"') {
            return Ok(Expression::Literal(Value::String(expr[1..expr.len()-1].to_string())));
        }
        
        // Number literal
        if let Ok(n) = expr.parse::<i64>() {
            return Ok(Expression::Literal(Value::Int(n)));
        }
        
        // Boolean
        if expr == "true" {
            return Ok(Expression::Literal(Value::Bool(true)));
        }
        if expr == "false" {
            return Ok(Expression::Literal(Value::Bool(false)));
        }
        
        // Variable
        if expr.starts_with('$') {
            return Ok(Expression::Variable(expr[1..].to_string()));
        }
        
        // Memory recall
        if expr.starts_with('@') {
            return Ok(Expression::Recall(expr[1..].to_string()));
        }
        
        // Tool call: !name/arity(args)
        if expr.starts_with('!') {
            let rest = &expr[1..];
            if let Some(idx) = rest.find('/') {
                let name = rest[..idx].to_string();
                let args_str = &rest[idx+1..];
                let args = if args_str.starts_with('(') && args_str.ends_with(')') {
                    let inner = &args_str[1..args_str.len()-1];
                    inner.split(',').map(|a| self.parse_expression(a)).collect::<Result<_, _>>()?
                } else {
                    vec![]
                };
                return Ok(Expression::ToolCall { name, arguments: args });
            }
        }
        
        // Function call: ^name/arity(args)
        if expr.starts_with('^') {
            let rest = &expr[1..];
            if let Some(idx) = rest.find('/') {
                let name = rest[..idx].to_string();
                let args_str = &rest[idx+1..];
                let args = if args_str.starts_with('(') && args_str.ends_with(')') {
                    let inner = &args_str[1..args_str.len()-1];
                    inner.split(',').map(|a| self.parse_expression(a)).collect::<Result<_, _>>()?
                } else {
                    vec![]
                };
                return Ok(Expression::FunctionCall { name, arguments: args });
            }
        }
        
        // Binary operation: left op right
        let ops = vec!["+", "-", "*", "/", "==", "!=", ">", "<", ">=", "<="];
        for op in ops {
            if let Some(idx) = expr.find(op) {
                if idx > 0 && idx + op.len() < expr.len() {
                    let left = self.parse_expression(&expr[..idx])?;
                    let right = self.parse_expression(&expr[idx+op.len()..])?;
                    return Ok(Expression::Binary {
                        left: Box::new(left),
                        operator: op.to_string(),
                        right: Box::new(right),
                    });
                }
            }
        }
        
        // Variable (default)
        Ok(Expression::Variable(expr.to_string()))
    }
}

/// Parse keyword source to AXL program
pub fn parse_keyword(source: &str) -> Result<Program, CompactParseError> {
    let mut parser = KeywordParser::new(source);
    parser.parse()
}
