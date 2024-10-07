use serde::Serialize;

/// Database wrapper for dictionarty.
pub trait Database<'a, T>
where 
    T: Serialize
{
    /// Gets single word from database.
    fn get(&'a self, query: String) -> Option<T>;

    /// Guesses possible queries form invalid query.
    fn suggest(&'a self, query: &String) -> Vec<&'a String>;

    /// Suggest words while searching.
    fn suggest_search(&'a self, query: &String) -> Vec<&'a String>;
}
