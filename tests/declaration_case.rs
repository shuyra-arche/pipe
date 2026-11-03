#[cfg(test)]
mod tests {
    use pipe_api::ENTRY;
    use pipe_declaration::{BUILD, UNIT};
    use unit::{Bootstrap, Dependency, Ending, Ready};

    #[test]
    fn unit_declaration_keeps_the_frozen_pipe_facts() {
        assert_eq!(UNIT.name, "pipe");
        assert_eq!(UNIT.loader.image, "pipe");
        assert!(matches!(UNIT.loader.space, env::ProgramKind::User));
        assert!(UNIT.loader.args.is_empty());
        assert_eq!(UNIT.loader.stack, 0);

        assert_eq!(UNIT.operator.namespace.len(), 1);
        let namespace = &UNIT.operator.namespace[0];
        assert_eq!(namespace.key, system_api::namespace::SERVICE);
        assert!(matches!(
            namespace.placement,
            unit::operator::Placement::Shared
        ));
        assert_eq!(namespace.path, "pipe");
        assert!(matches!(namespace.scope, unit::operator::Scope::Subtree));
        assert_eq!(
            namespace.ops,
            system_api::operator::Ops::PART
                .union(system_api::operator::Ops::UNPART)
                .union(system_api::operator::Ops::BIND)
                .union(system_api::operator::Ops::UNBIND)
                .union(system_api::operator::Ops::FIND)
        );
        assert_eq!(
            namespace.permit,
            Some(system_api::operator::Permit::Public)
        );
        assert_eq!(namespace.source_mark, None);

        assert_eq!(namespace.bindings.len(), 1);
        let binding = &namespace.bindings[0];
        assert_eq!(binding.name, "create");
        assert_eq!(binding.mark, ENTRY);
        assert_eq!(binding.view.bits(), env::Access::STORE.bits());
        assert_eq!(binding.permit, system_api::operator::Permit::Public);

        assert!(UNIT.identity.aliases.is_empty());
        assert!(UNIT.identity.grants.is_empty());

        let after = UNIT.control.after.expect("pipe is listed");
        assert_eq!(after.len(), 2);
        assert!(matches!(
            after[0],
            Dependency::Bootstrap(Bootstrap::Operator)
        ));
        assert!(matches!(
            after[1],
            Dependency::Bootstrap(Bootstrap::Identity)
        ));
        assert!(matches!(
            UNIT.control.ready,
            Ready::Signal(mark) if mark == unit::READY_MARK
        ));
        assert!(matches!(UNIT.control.ending, Some(Ending::Resident)));
        assert!(UNIT.control.construction.is_empty());
        assert!(UNIT.control.retirement.is_empty());
        assert!(UNIT.control.supervision.is_none());
        assert!(UNIT.supply.bindings.is_empty());
        assert!(UNIT.valid());
    }

    #[test]
    fn image_build_names_the_provider_server_target() {
        assert_eq!(BUILD.image, "pipe");
        assert_eq!(BUILD.manifest, "server/Cargo.toml");
        assert_eq!(BUILD.package, "pipe-server");
        assert_eq!(BUILD.bin, "pipe");
        assert!(BUILD.features.is_empty());
        assert_eq!(BUILD.target, "riscv64gc-unknown-none-elf");
        assert_eq!(BUILD.profile, "release");
        assert!(BUILD.default_features);
    }
}
