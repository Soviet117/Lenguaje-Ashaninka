#![allow(dead_code)]
use std::fs;
use std::env;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    // Palabras reservadas
    AsankiKeyword,
    ToyKeyword,
    KametsaKeyword,
    PaiKeyword,
    PaiTeKeyword,
    KamKeyword,
    KaraKeyword,
    TasKeyword,
    PawKeyword,
    IroqKeyword,
    IrokKeyword,
    PawaKeyword,
    ToyaKeyword,
    
    // Valores
    Asanki(String),
    Toy(u32),
    Kametsa(f32),
    
    // Otros
    Variable(String),
    Equal,
    Semicolon,
    Comma,
    LBrace,
    RBrace,
    LParen,
    RParen,
}

struct Lexer {
    texto: Vec<char>,
    pos: usize,
}

impl Lexer {
    fn nuevo(entrada: &str) -> Self {
        Lexer {
            texto: entrada.chars().collect(),
            pos: 0,
        }
    }

    fn obtener_token(&mut self) -> Option<Token> {
        self.saltar_espacios();

        if self.esta_al_final() {
            return None;
        }

        let c = self.texto[self.pos];
        match c {
            '=' => { self.pos += 1; return Some(Token::Equal); }
            ';' => { self.pos += 1; return Some(Token::Semicolon); }
            ',' => { self.pos += 1; return Some(Token::Comma); }
            '{' => { self.pos += 1; return Some(Token::LBrace); }
            '}' => { self.pos += 1; return Some(Token::RBrace); }
            '(' => { self.pos += 1; return Some(Token::LParen); }
            ')' => { self.pos += 1; return Some(Token::RParen); }
            _ => {}
        }

        let inicio = self.pos;
        self.avanzar_mientras_sea_valido();

        if inicio == self.pos {
            self.pos += 1; // saltar carácter no reconocido
            return self.obtener_token();
        }

        let palabra = self.obtener_palabra(inicio);
        self.clasificar_token(palabra)
    }

    fn saltar_espacios(&mut self) {
        while self.pos < self.texto.len() && self.texto[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn esta_al_final(&self) -> bool {
        self.pos >= self.texto.len()
    }

    fn avanzar_mientras_sea_valido(&mut self) {
        while self.pos < self.texto.len() && self.es_caracter_valido(self.texto[self.pos]) {
            self.pos += 1;
        }
    }

    fn es_caracter_valido(&self, c: char) -> bool {
        c.is_alphanumeric() || c == '_' || c == '\'' || c == '.'
    }

    fn obtener_palabra(&self, inicio: usize) -> String {
        self.texto[inicio..self.pos].iter().collect()
    }

    fn clasificar_token(&mut self, palabra: String) -> Option<Token> {
        if self.termina_con_comilla() {
            return Some(Token::Asanki(palabra));
        }

        match palabra.as_str() {
            "toy" => return Some(Token::ToyKeyword),
            "kametsa" => return Some(Token::KametsaKeyword),
            "asanki" => return Some(Token::AsankiKeyword),
            "pai" => return Some(Token::PaiKeyword),
            "pai_te" => return Some(Token::PaiTeKeyword),
            "kam" => return Some(Token::KamKeyword),
            "kara" => return Some(Token::KaraKeyword),
            "tas" => return Some(Token::TasKeyword),
            "paw" => return Some(Token::PawKeyword),
            "iroq" => return Some(Token::IroqKeyword),
            "irok" => return Some(Token::IrokKeyword),
            "pawa" => return Some(Token::PawaKeyword),
            "toya" => return Some(Token::ToyaKeyword),
            _ => {}
        }

        if palabra.contains('.') {
            let valor = palabra.parse::<f32>().expect("Error al parsear flotante");
            Some(Token::Kametsa(valor))
        } else if palabra.chars().all(|c| c.is_ascii_digit()) {
            let valor = palabra.parse::<u32>().expect("Error al parsear entero");
            Some(Token::Toy(valor))
        } else {
            Some(Token::Variable(palabra))
        }
    }

    fn termina_con_comilla(&self) -> bool {
        self.pos > 0 && self.texto[self.pos - 1] == '\''
    }
}

// ==========================================
// ESTRUCTURAS DEL AST
// ==========================================

#[derive(Debug)]
enum Tipo {
    Asanki,
    Toy,
    Kametsa,
}

#[derive(Debug)]
enum Expresion {
    Variable(String),
    Asanki(String),
    Toy(u32),
    Kametsa(f32),
}

#[derive(Debug)]
enum Sentencia {
    // Declaracion: Tipo de dato, nombre de la variable y su valor inicial
    Declaracion(Tipo, String, Expresion),
    // Asignacion: Nombre de variable y su nuevo valor
    Asignacion(String, Expresion),
    // Funcion: Nombre y la lista de sentencias dentro de sus llaves
    Funcion(String, Vec<Sentencia>),
    // Llamada: Imprimir (pawa) una variable
    Llamada(String),
}

#[derive(Debug)]
struct Programa {
    sentencias: Vec<Sentencia>,
}

// ==========================================
// PARSER CON VALIDACIÓN
// ==========================================

struct Parser {
    lexer: Lexer,
    token_actual: Option<Token>,
}

impl Parser {
    fn nuevo(mut lexer: Lexer) -> Self {
        let token_actual = lexer.obtener_token();
        Parser { lexer, token_actual }
    }

    // Avanza al siguiente token generado por el lexer
    fn avanzar(&mut self) {
        self.token_actual = self.lexer.obtener_token();
    }

    fn coincidir(&mut self, token_esperado: Token) -> bool {
        if let Some(ref t) = self.token_actual {
            if std::mem::discriminant(t) == std::mem::discriminant(&token_esperado) {
                self.avanzar();
                return true;
            }
        }
        false
    }

    // Parsea todo el texto y retorna Result. Si encuentra un error, se detiene inmediatamente.
    fn parsear_programa(&mut self) -> Result<Programa, String> {
        let mut sentencias = Vec::new();
        while self.token_actual.is_some() {
            let sentencia = self.parsear_sentencia()?;
            sentencias.push(sentencia);
        }
        Ok(Programa { sentencias })
    }

    // Valida la sintaxis del lenguaje
    fn parsear_sentencia(&mut self) -> Result<Sentencia, String> {
        let token = self.token_actual.clone().ok_or("Fin de archivo inesperado")?;

        match token {
            // 1. Declaraciones de variables: asanki/toy/kametsa nombre = valor
            Token::AsankiKeyword | Token::ToyKeyword | Token::KametsaKeyword => {
                let tipo = match token {
                    Token::AsankiKeyword => Tipo::Asanki,
                    Token::ToyKeyword => Tipo::Toy,
                    Token::KametsaKeyword => Tipo::Kametsa,
                    _ => unreachable!(),
                };
                self.avanzar(); // Saltamos el tipo de variable

                // Esperamos un identificador (Variable)
                let nombre = match self.token_actual.clone() {
                    Some(Token::Variable(n)) => {
                        self.avanzar();
                        n
                    }
                    Some(t) => return Err(format!("Se esperaba el nombre de la variable, pero se encontró: {:?}", t)),
                    None => return Err("Se esperaba el nombre de la variable, pero se llegó al fin del archivo".to_string()),
                };

                // Esperamos el signo '='
                if !self.coincidir(Token::Equal) {
                    return Err(format!(
                        "Error en declaración de '{}': se esperaba el símbolo '=' después del nombre, pero se encontró: {:?}",
                        nombre, self.token_actual
                    ));
                }

                // Esperamos el valor de la variable
                let valor = self.parsear_expresion()?;
                
                // Permitir punto y coma ';' opcional
                let _ = self.coincidir(Token::Semicolon);
                
                Ok(Sentencia::Declaracion(tipo, nombre, valor))
            }

            // 4. Asignaciones directas a variables: nombre = valor
            Token::Variable(nombre) => {
                self.avanzar(); // Saltamos el nombre

                // Esperamos el signo '='
                if !self.coincidir(Token::Equal) {
                    return Err(format!(
                        "Error en asignación: se esperaba '=' después de '{}', pero se encontró: {:?}",
                        nombre, self.token_actual
                    ));
                }

                let valor = self.parsear_expresion()?;
                
                let _ = self.coincidir(Token::Semicolon);
                
                Ok(Sentencia::Asignacion(nombre, valor))
            }

            // 2. Funciones: tas nombre { cuerpo }
            Token::TasKeyword => {
                self.avanzar(); // Saltamos 'tas'
                
                // Esperamos el nombre de la función
                let nombre = match self.token_actual.clone() {
                    Some(Token::Variable(n)) => {
                        self.avanzar();
                        n
                    }
                    Some(t) => return Err(format!("Se esperaba el nombre de la función, pero se encontró: {:?}", t)),
                    None => return Err("Se esperaba el nombre de la función, pero se llegó al fin del archivo".to_string()),
                };

                // Esperamos llave abierta '{'
                if !self.coincidir(Token::LBrace) {
                    return Err(format!("Se esperaba '{{' al iniciar la función '{}', pero se encontró: {:?}", nombre, self.token_actual));
                }

                // Parseamos todo lo que está dentro de las llaves
                let mut cuerpo = Vec::new();
                while self.token_actual.is_some() && self.token_actual != Some(Token::RBrace) {
                    cuerpo.push(self.parsear_sentencia()?);
                }

                // Esperamos llave cerrada '}'
                if !self.coincidir(Token::RBrace) {
                    return Err(format!("Se esperaba '}}' al finalizar la función '{}', pero se llegó al fin del archivo", nombre));
                }
                
                Ok(Sentencia::Funcion(nombre, cuerpo))
            }

            // 3. Llamadas: pawa(variable)
            Token::PawaKeyword => {
                self.avanzar(); // Saltamos 'pawa'
                
                // Esperamos '('
                if !self.coincidir(Token::LParen) {
                    return Err(format!("Se esperaba '(' después de 'pawa', pero se encontró: {:?}", self.token_actual));
                }
                
                // Esperamos la variable
                let variable = match self.token_actual.clone() {
                    Some(Token::Variable(v)) => {
                        self.avanzar();
                        v
                    }
                    Some(t) => return Err(format!("Se esperaba una variable dentro de pawa(), pero se encontró: {:?}", t)),
                    None => return Err("Se esperaba una variable dentro de pawa(), pero se llegó al fin del archivo".to_string()),
                };

                // Esperamos ')'
                if !self.coincidir(Token::RParen) {
                    return Err(format!("Se esperaba ')' para cerrar la llamada a pawa, pero se encontró: {:?}", self.token_actual));
                }

                let _ = self.coincidir(Token::Semicolon);

                Ok(Sentencia::Llamada(variable))
            }

            _ => Err(format!("Token no esperado al inicio de sentencia: {:?}", token)),
        }
    }

    // Parsea los valores a la derecha de '='
    fn parsear_expresion(&mut self) -> Result<Expresion, String> {
        let token = self.token_actual.clone().ok_or("Se esperaba un valor, pero se llegó al fin del archivo")?;
        let expresion = match token {
            Token::Variable(n) => Expresion::Variable(n),
            Token::Asanki(s) => Expresion::Asanki(s),
            Token::Toy(v) => Expresion::Toy(v),
            Token::Kametsa(v) => Expresion::Kametsa(v),
            _ => return Err(format!("Se esperaba un valor o variable, pero se encontró: {:?}", token)),
        };
        self.avanzar();
        Ok(expresion)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let codigo = if args.len() > 1 {
        fs::read_to_string(&args[1]).expect("No se pudo leer el archivo")
    } else {
        "asanki stri = 'Soviet'\ntoy ent = 4\nkametsa flo = 1.2\ntoy nose = 5\n\ntoy dos 7 =\n\ntas suma{\n    toy x = 2\n    pawa(x)\n}".to_string()
    };

    println!("Código a analizar:\n{}", codigo);
    println!("------------------------------------------------");

    let lexer = Lexer::nuevo(&codigo);
    let mut parser = Parser::nuevo(lexer);

    match parser.parsear_programa() {
        Ok(ast) => {
            println!("AST GENERADO CON ÉXITO:\n{:#?}", ast);
        }
        Err(e) => {
            println!("ERROR SINTÁCTICO: {}", e);
        }
    }
}
