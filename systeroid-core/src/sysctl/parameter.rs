use crate::config::Config;
use crate::error::Result;
use crate::sysctl::display::DisplayType;
use crate::sysctl::section::Section;
use colored::*;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use sysctl::{Ctl, Sysctl as SysctlImpl};

/// Representation of a kernel parameter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Parameter {
    /// Name of the kernel parameter.
    pub name: String,
    /// Value of the kernel parameter.
    #[serde(skip)]
    pub value: String,
    /// Description of the kernel parameter
    pub description: Option<String>,
    /// Section of the kernel parameter.
    pub section: Section,
    /// Documentation path.
    pub docs_path: PathBuf,
    /// Title of the kernel parameter taken from the documentation.
    pub docs_title: String,
}

impl<'a> TryFrom<&'a Ctl> for Parameter {
    type Error = crate::error::Error;
    fn try_from(ctl: &'a Ctl) -> Result<Self> {
        Ok(Parameter {
            name: ctl.name()?,
            value: ctl.value_string()?,
            description: ctl
                .description()
                .ok()
                .and_then(|v| (v == "[N/A]").then(|| None)?),
            section: Section::from(ctl.name()?),
            docs_path: PathBuf::new(),
            docs_title: String::new(),
        })
    }
}

impl Parameter {
    /// Returns the absolute name of the parameter, without the sections.
    pub fn absolute_name(&self) -> Option<&str> {
        self.name.split('.').collect::<Vec<&str>>().last().copied()
    }

    /// Returns the parameter name with corresponding section colors.
    pub fn colored_name(&self, config: &Config) -> String {
        let section_color = *(config
            .section_colors
            .get(&self.section)
            .unwrap_or(&config.default_color));
        let fields = self.name.split('.').collect::<Vec<&str>>();
        fields
            .iter()
            .enumerate()
            .fold(String::new(), |mut result, (i, v)| {
                if i != fields.len() - 1 {
                    result += &format!(
                        "{}{}",
                        v.color(section_color),
                        ".".color(config.default_color)
                    );
                } else {
                    result += v;
                }
                result
            })
    }

    /// Returns the components of the parameter to contruct a [`Tree`].
    ///
    /// [`Tree`]: crate::tree::Tree
    pub fn get_tree_components(&self, config: &Config) -> Vec<String> {
        let section_color = *(config
            .section_colors
            .get(&self.section)
            .unwrap_or(&config.default_color));
        let mut components = self
            .name
            .split('.')
            .map(String::from)
            .collect::<Vec<String>>();
        let total_components = components.len();
        components
            .iter_mut()
            .enumerate()
            .for_each(|(i, component)| {
                if i != total_components - 1 {
                    *component = component.color(section_color).to_string();
                } else {
                    *component = format!(
                        "{} {} {}",
                        component,
                        "=".color(config.default_color),
                        self.value.replace('\n', " ").bold()
                    );
                }
            });
        components
    }

    /// Prints the kernel parameter to given output.
    pub fn display_value<Output: Write>(&self, config: &Config, output: &mut Output) -> Result<()> {
        match config.display_type {
            DisplayType::Name => {
                writeln!(output, "{}", self.colored_name(config))?;
            }
            DisplayType::Value => {
                writeln!(output, "{}", self.value.bold())?;
            }
            DisplayType::Binary => {
                write!(output, "{}", self.value.bold())?;
            }
            DisplayType::Default => {
                writeln!(
                    output,
                    "{} {} {}",
                    self.colored_name(config),
                    "=".color(config.default_color),
                    self.value.bold(),
                )?;
            }
        }
        Ok(())
    }

    /// Returns the parameter documentation if it exists.
    pub fn get_documentation(&self) -> Option<String> {
        self.description.as_ref().map(|description| {
            format!(
                "{}\n{}\n{}\n-\nParameter: {}\nReference: {}",
                self.docs_title,
                "=".repeat(self.docs_title.len()),
                description,
                self.name,
                self.docs_path.to_string_lossy()
            )
        })
    }

    /// Prints the description of the kernel parameter to the given output.
    pub fn display_documentation<Output: Write>(&self, output: &mut Output) -> Result<()> {
        if let Some(documentation) = self.get_documentation() {
            writeln!(output, "{}\n", documentation)?;
        } else {
            writeln!(output, "No documentation available")?;
        }
        Ok(())
    }

    /// Sets a new value for the kernel parameter.
    pub fn update_value<Output: Write>(
        &mut self,
        new_value: &str,
        config: &Config,
        output: &mut Output,
    ) -> Result<()> {
        let ctl = Ctl::new(&self.name)?;
        let new_value = ctl.set_value_string(new_value)?;
        self.value = new_value;
        if !config.quiet {
            self.display_value(config, output)?;
        }
        Ok(())
    }
}
