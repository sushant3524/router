use std::collections::HashMap;

use apollo_compiler::ast::Argument;
use apollo_compiler::ast::Directive;
use apollo_compiler::name;
use apollo_compiler::schema::Component;
use apollo_compiler::schema::EnumType;
use apollo_compiler::Name;
use apollo_compiler::Node;
use lazy_static::lazy_static;

use crate::error::FederationError;
use crate::link::spec::Identity;
use crate::link::spec::Url;
use crate::link::spec::Version;
use crate::link::spec_definition::SpecDefinition;
use crate::link::spec_definition::SpecDefinitions;
use crate::schema::position::EnumTypeDefinitionPosition;
use crate::schema::FederationSchema;

pub(crate) const COST_DIRECTIVE_NAME_IN_SPEC: Name = name!("cost");
pub(crate) const COST_DIRECTIVE_NAME_DEFAULT: Name = name!("federation__cost");
pub(crate) const COST_WEIGHT_ARGUMENT_NAME: Name = name!("weight");

pub(crate) const LIST_SIZE_DIRECTIVE_NAME_IN_SPEC: Name = name!("listSize");
pub(crate) const LIST_SIZE_DIRECTIVE_NAME_DEFAULT: Name = name!("federation__listSize");
pub(crate) const LIST_SIZE_ASSUMED_SIZE_ARGUMENT_NAME: Name = name!("assumedSize");
pub(crate) const LIST_SIZE_SLICING_ARGUMENTS_ARGUMENT_NAME: Name = name!("slicingArguments");
pub(crate) const LIST_SIZE_SIZED_FIELDS_ARGUMENT_NAME: Name = name!("sizedFields");
pub(crate) const LIST_SIZE_REQUIRE_ONE_SLICING_ARGUMENT_ARGUMENT_NAME: Name =
    name!("requireOneSlicingArgument");

#[derive(Clone)]
pub(crate) struct CostSpecDefinition {
    url: Url,
    minimum_federation_version: Option<Version>,
}

impl CostSpecDefinition {
    pub(crate) fn new(version: Version, minimum_federation_version: Option<Version>) -> Self {
        Self {
            url: Url {
                identity: Identity::cost_identity(),
                version,
            },
            minimum_federation_version,
        }
    }

    pub(crate) fn cost_directive(
        &self,
        schema: &FederationSchema,
        arguments: Vec<Node<Argument>>,
    ) -> Result<Directive, FederationError> {
        let name = self
            .directive_name_in_schema(schema, &COST_DIRECTIVE_NAME_IN_SPEC)?
            .unwrap_or(COST_DIRECTIVE_NAME_DEFAULT);

        Ok(Directive { name, arguments })
    }

    pub(crate) fn list_size_directive(
        &self,
        schema: &FederationSchema,
        arguments: Vec<Node<Argument>>,
    ) -> Result<Directive, FederationError> {
        let name = self
            .directive_name_in_schema(schema, &LIST_SIZE_DIRECTIVE_NAME_IN_SPEC)?
            .unwrap_or(LIST_SIZE_DIRECTIVE_NAME_DEFAULT);

        Ok(Directive { name, arguments })
    }

    pub(crate) fn propagate_demand_control_directives(
        &self,
        subgraph_schema: &FederationSchema,
        source: &apollo_compiler::ast::DirectiveList,
        dest: &mut apollo_compiler::ast::DirectiveList,
        original_directive_names: &HashMap<Name, Name>,
    ) -> Result<(), FederationError> {
        let cost_directive_name = original_directive_names.get(&COST_DIRECTIVE_NAME_IN_SPEC);
        if let Some(cost_directive) = source.get(
            cost_directive_name
                .unwrap_or(&COST_DIRECTIVE_NAME_IN_SPEC)
                .as_str(),
        ) {
            dest.push(Node::new(self.cost_directive(
                subgraph_schema,
                cost_directive.arguments.clone(),
            )?));
        }

        let list_size_directive_name =
            original_directive_names.get(&LIST_SIZE_DIRECTIVE_NAME_IN_SPEC);
        if let Some(list_size_directive) = source.get(
            list_size_directive_name
                .unwrap_or(&LIST_SIZE_DIRECTIVE_NAME_IN_SPEC)
                .as_str(),
        ) {
            dest.push(Node::new(self.list_size_directive(
                subgraph_schema,
                list_size_directive.arguments.clone(),
            )?));
        }

        Ok(())
    }

    pub(crate) fn propagate_demand_control_directives_for_enum(
        &self,
        subgraph_schema: &mut FederationSchema,
        source: &Node<EnumType>,
        dest: &EnumTypeDefinitionPosition,
        original_directive_names: &HashMap<Name, Name>,
    ) -> Result<(), FederationError> {
        let cost_directive_name = original_directive_names.get(&COST_DIRECTIVE_NAME_IN_SPEC);
        if let Some(cost_directive) = source.directives.get(
            cost_directive_name
                .unwrap_or(&COST_DIRECTIVE_NAME_IN_SPEC)
                .as_str(),
        ) {
            dest.insert_directive(
                subgraph_schema,
                Component::from(
                    self.cost_directive(subgraph_schema, cost_directive.arguments.clone())?,
                ),
            )?;
        }

        let list_size_directive_name =
            original_directive_names.get(&LIST_SIZE_DIRECTIVE_NAME_IN_SPEC);
        if let Some(list_size_directive) = source.directives.get(
            list_size_directive_name
                .unwrap_or(&LIST_SIZE_DIRECTIVE_NAME_IN_SPEC)
                .as_str(),
        ) {
            dest.insert_directive(
                subgraph_schema,
                Component::from(
                    self.list_size_directive(
                        subgraph_schema,
                        list_size_directive.arguments.clone(),
                    )?,
                ),
            )?;
        }

        Ok(())
    }
}

impl SpecDefinition for CostSpecDefinition {
    fn url(&self) -> &Url {
        &self.url
    }

    fn minimum_federation_version(&self) -> Option<&Version> {
        self.minimum_federation_version.as_ref()
    }
}

lazy_static! {
    pub(crate) static ref COST_VERSIONS: SpecDefinitions<CostSpecDefinition> = {
        let mut definitions = SpecDefinitions::new(Identity::cost_identity());
        definitions.add(CostSpecDefinition::new(
            Version { major: 0, minor: 1 },
            Some(Version { major: 2, minor: 9 }),
        ));
        definitions
    };
}
