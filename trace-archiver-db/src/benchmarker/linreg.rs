#[doc(hidden)]
use anyhow::Result;
use linregress::{FormulaRegressionBuilder, RegressionDataBuilder, RegressionData, RegressionModel};

pub fn create_data(vars: Vec<(&str,Vec<f64>)>) -> Result<RegressionData,anyhow::Error>
{
    match RegressionDataBuilder::new().build_from(vars) {
        Ok(data) => Ok(data),
        Err(e) => Err(anyhow::Error::new(e)),
    }
}

pub fn create_model(data : &RegressionData, formula: &str) -> Result<RegressionModel,linregress::Error>
{
    FormulaRegressionBuilder::new()
    .data(data)
    .formula(formula)
    .fit()
}

/// Print the regression model
pub fn print_summary_statistics(model : &RegressionModel, name: &str) {
    println!("Mutlilinear Regression: {name}");
    
    print!("intercept = ");
    println_multilin_reg_coef(&model,0);
    for i in 1..=3 {
        print!("\tParameter '{0}': Slope = ",model.regressor_names()[i - 1]);
        println_multilin_reg_coef(&model, i);
    }
    println!();
}

fn println_multilin_reg_coef(model : &RegressionModel, index : usize) {
    print!("{0:8.5} us )", model.parameters()[index] / 1000.0);
    print_multilin_reg_coef_stats(model, index);
    println!();
}

fn print_multilin_reg_coef_stats(model : &RegressionModel, index : usize) {
    print!("(p = {0:8.3e}, std-err = {1:8.5})", model.p_values()[index], model.se()[index] / 1000.0);
}
