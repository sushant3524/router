// No panics allowed in this module
// The expansion code is called with potentially invalid schemas during the
// authoring process and we can't panic in the language server.
#![cfg_attr(
    not(test),
    deny(
        clippy::exit,
        clippy::panic,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::unimplemented,
        clippy::todo
    )
)]

mod directives;
pub(crate) mod schema;
mod type_and_directive_specifications;

use std::sync::Arc;

use apollo_compiler::ast::Directive;
use apollo_compiler::name;
use apollo_compiler::Name;
use apollo_compiler::Schema;
pub(crate) use directives::extract_connect_directive_arguments;
pub(crate) use directives::extract_source_directive_arguments;
use lazy_static::lazy_static;
pub(crate) use schema::ConnectHTTPArguments;
pub(crate) use schema::HTTPHeaderOption;
pub(crate) use schema::SourceHTTPArguments;

use self::schema::CONNECT_DIRECTIVE_NAME_IN_SPEC;
use self::schema::SOURCE_DIRECTIVE_NAME_IN_SPEC;
use crate::error::FederationError;
use crate::link::database::links_metadata;
use crate::link::spec::Identity;
use crate::link::spec::Url;
use crate::link::spec::Version;
use crate::link::spec::APOLLO_SPEC_DOMAIN;
use crate::link::spec_definition::SpecDefinition;
use crate::link::spec_definition::SpecDefinitions;
use crate::link::Link;
use crate::schema::FederationSchema;

pub(crate) struct ConnectSpecDefinition {
    url: Url,
    minimum_federation_version: Option<Version>,
}

impl ConnectSpecDefinition {
    pub(crate) fn new(version: Version, minimum_federation_version: Option<Version>) -> Self {
        Self {
            url: Url {
                identity: Self::identity(),
                version,
            },
            minimum_federation_version,
        }
    }

    pub(crate) fn from_directive(
        directive: &Directive,
    ) -> Result<Option<&'static Self>, FederationError> {
        let Some(url) = directive.argument_by_name("url").and_then(|a| a.as_str()) else {
            return Ok(None);
        };

        let url: Url = url.parse()?;

        Ok(CONNECT_VERSIONS.find(&url.version))
    }

    pub(crate) fn identity() -> Identity {
        Identity {
            domain: APOLLO_SPEC_DOMAIN.to_string(),
            name: name!("connect"),
        }
    }

    pub(crate) fn get_from_schema(
        schema: &Schema,
    ) -> Result<Option<(&'static ConnectSpecDefinition, Arc<Link>)>, FederationError> {
        let metadata = links_metadata(schema)?;
        Ok(metadata
            .as_ref()
            .and_then(|metadata| metadata.for_identity(&ConnectSpecDefinition::identity()))
            .and_then(|link| {
                CONNECT_VERSIONS
                    .find(&link.url.version)
                    .map(|v| (v, link.clone()))
            }))
    }

    pub(crate) fn get_from_federation_schema(
        schema: &FederationSchema,
    ) -> Result<Option<&'static ConnectSpecDefinition>, FederationError> {
        Ok(schema
            .metadata()
            .as_ref()
            .and_then(|metadata| metadata.for_identity(&ConnectSpecDefinition::identity()))
            .and_then(|link| CONNECT_VERSIONS.find(&link.url.version)))
    }

    pub(crate) fn check_or_add(schema: &mut FederationSchema) -> Result<(), FederationError> {
        let Some(link) = schema
            .metadata()
            .and_then(|metadata| metadata.for_identity(&ConnectSpecDefinition::identity()))
        else {
            return Ok(());
        };

        type_and_directive_specifications::check_or_add(&link, schema)
    }

    pub(crate) fn source_directive_name(link: &Link) -> Name {
        link.directive_name_in_schema(&SOURCE_DIRECTIVE_NAME_IN_SPEC)
    }

    pub(crate) fn connect_directive_name(link: &Link) -> Name {
        link.directive_name_in_schema(&CONNECT_DIRECTIVE_NAME_IN_SPEC)
    }
}

impl SpecDefinition for ConnectSpecDefinition {
    fn url(&self) -> &Url {
        &self.url
    }

    fn minimum_federation_version(&self) -> Option<&Version> {
        self.minimum_federation_version.as_ref()
    }
}

lazy_static! {
    pub(crate) static ref CONNECT_VERSIONS: SpecDefinitions<ConnectSpecDefinition> = {
        let mut definitions = SpecDefinitions::new(ConnectSpecDefinition::identity());

        definitions.add(ConnectSpecDefinition::new(
            Version { major: 0, minor: 1 },
            Some(Version { major: 2, minor: 8 }), // TODO
        ));

        definitions
    };
}
