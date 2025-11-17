use crate::database::{orm::{DatabaseBackend, ColumnTrait, Value}, query::{SimpleExpr, Expr}};


/// Extension trait for JSON column operations
pub trait JsonColumnExt {
    fn json_key_eq<K, V>(&self, backend: &DatabaseBackend, key: K, value: V) -> SimpleExpr
    where
        K: Into<String>,
        V: Into<Value>;
}

impl<C: ColumnTrait> JsonColumnExt for C {
    fn json_key_eq<K, V>(&self, backend: &DatabaseBackend, key: K, value: V) -> SimpleExpr
    where 
        K: Into<String>,
        V: Into<Value>,
    {
        match backend {
            DatabaseBackend::Sqlite => 
                Expr::cust_with_values(format!("json_extract({}, ?)", self.to_string()), vec![format!("$.{}", key.into())])
                    .eq(value),
            DatabaseBackend::Postgres =>
                Expr::cust_with_values(format!("{} ->> $1", self.to_string()), vec![key.into()])
                    .eq(value),
            _ => panic!("Unsupported backend for JSON column extension"),
        }
    }
}