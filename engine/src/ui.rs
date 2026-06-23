use std::error::Error;

use dices_values::{Value, injected::call::ManualError, string::ValueString};

/// Handler to the user interface
pub trait Ui {
    type PrintError: Into<Box<dyn Error>>;
    /// Print a value exactly as an expression result
    fn print(&self, value: impl Into<Value>) -> Result<(), Self::PrintError>;

    /// Print a string
    fn print_str<V: AsRef<str> + Into<ValueString>>(
        &self,
        value: V,
    ) -> Result<(), Self::PrintError>;

    /// Print a markdown string
    fn print_md<V: AsRef<str> + Into<ValueString>>(&self, value: V)
    -> Result<(), Self::PrintError>;

    /// Display a manual page
    ///
    /// Returns only when the user exit the page
    fn manual(&self, page: impl Into<ValueString>) -> Result<(), ManualError>;
}
