//! Shell completion script generator for `syn` and `syntrop`.

/// Bash completion script for `syn` and `syntrop`.
pub const BASH_COMPLETION: &str = r#"# bash completion for syn and syntrop
_syn() {
    local cur prev words cword
    _init_completion -n : 2>/dev/null || {
        cur="${COMP_WORDS[COMP_CWORD]}"
        prev="${COMP_WORDS[COMP_CWORD-1]}"
        words=("${COMP_WORDS[@]}")
        cword=$COMP_CWORD
    }
    local namespaces="router runtime store hardware context tools fleet system"
    local subcommands="pull setup admin companion talk telemetry tune visual audio video decide audit prompt completions"
    local flags="--help -h --version -V -e --effort"
    local effort_tiers="none low med high max"
    local families="qwen granite gemma phi"
    local policies="balanced throughput low-latency"
    local shells="bash zsh fish"

    if [[ "$prev" == "-e" || "$prev" == "--effort" ]]; then
        COMPREPLY=( $(compgen -W "${effort_tiers}" -- "$cur") )
        return 0
    fi
    if [[ "$cur" == --effort=* ]]; then
        local val="${cur#--effort=}"
        COMPREPLY=( $(compgen -P "--effort=" -W "${effort_tiers}" -- "$val") )
        return 0
    fi

    local start_idx=0
    for (( idx=0; idx <= cword; idx++ )); do
        if [[ "${words[idx]}" == "syn" || "${words[idx]}" == "syntrop" || "${words[idx]}" == */syn || "${words[idx]}" == */syntrop ]]; then
            start_idx=$((idx + 1))
            break
        fi
    done
    if [[ $start_idx -eq 0 ]]; then start_idx=1; fi

    local cmd="" cmd_idx=0
    for (( i=start_idx; i < cword; i++ )); do
        local w="${words[i]}"
        if [[ "$w" == "-e" || "$w" == "--effort" ]]; then ((i++)); continue; fi
        if [[ "$w" == --effort=* || "$w" == -e* ]]; then continue; fi
        if [[ "$w" != -* ]]; then cmd="$w"; cmd_idx=$i; break; fi
    done

    if [[ "$cur" == -* && -z "$cmd" ]]; then
        COMPREPLY=( $(compgen -W "${flags}" -- "$cur") )
        return 0
    fi
    if [[ -z "$cmd" || $cword -eq $cmd_idx ]]; then
        COMPREPLY=( $(compgen -W "${namespaces} ${subcommands} ${flags}" -- "$cur") )
        return 0
    fi

    case "$cmd" in
        completions|completion) COMPREPLY=( $(compgen -W "${shells}" -- "$cur") ) ;;
        setup)
            if [[ "$prev" == "--family" || "$prev" == "-f" ]]; then
                COMPREPLY=( $(compgen -W "${families}" -- "$cur") )
            else
                COMPREPLY=( $(compgen -W "--family --dry-run" -- "$cur") )
            fi ;;
        tune)
            if [[ "$prev" == "--policy" || "$prev" == "-p" ]]; then
                COMPREPLY=( $(compgen -W "${policies}" -- "$cur") )
            else
                COMPREPLY=( $(compgen -W "--policy -p status ${policies}" -- "$cur") )
            fi ;;
        telemetry) COMPREPLY=( $(compgen -W "status tune --json" -- "$cur") ) ;;
        admin)
            if [[ $cword -eq $((cmd_idx + 1)) ]]; then
                COMPREPLY=( $(compgen -W "status remediate rollback audit lockout" -- "$cur") )
            else
                local sub="${words[cmd_idx + 1]}"
                if [[ "$sub" == "lockout" ]]; then
                    COMPREPLY=( $(compgen -W "reset" -- "$cur") )
                elif [[ "$sub" == "remediate" ]]; then
                    COMPREPLY=( $(compgen -W "--recipe --dry-run" -- "$cur") )
                elif [[ "$sub" == "audit" ]]; then
                    COMPREPLY=( $(compgen -W "--unit --limit -u -n" -- "$cur") )
                fi
            fi ;;
        companion) COMPREPLY=( $(compgen -W "status ask execute listen stop talk" -- "$cur") ) ;;
        router) COMPREPLY=( $(compgen -W "ask setup models default test" -- "$cur") ) ;;
        runtime) COMPREPLY=( $(compgen -W "generate status models" -- "$cur") ) ;;
        store) COMPREPLY=( $(compgen -W "list import prune inspect" -- "$cur") ) ;;
        hardware) COMPREPLY=( $(compgen -W "planes leases status" -- "$cur") ) ;;
        context) COMPREPLY=( $(compgen -W "history drift status" -- "$cur") ) ;;
        tools) COMPREPLY=( $(compgen -W "list run rollback status" -- "$cur") ) ;;
        fleet) COMPREPLY=( $(compgen -W "status health forensics" -- "$cur") ) ;;
        system) COMPREPLY=( $(compgen -W "units triage version status" -- "$cur") ) ;;
        visual|audio|video) COMPREPLY=( $(compgen -W "generate" -- "$cur") ) ;;
    esac
}
complete -F _syn syn syntrop
"#;

/// Zsh completion script for `syn` and `syntrop`.
pub const ZSH_COMPLETION: &str = r#"#compdef syn syntrop
_syn() {
    local -a namespaces commands shells effort_tiers families policies
    namespaces=(
        'router:Talk to LLMs: ask, setup, models, default, test'
        'runtime:Run local models directly'
        'store:Model file storage: list, import, prune'
        'hardware:GPU/accelerator planes and leases'
        'context:System history and config drift'
        'tools:Sandboxed repair tools and rollback'
        'fleet:Fleet health and failure forensics'
        'system:Units, triage, and suite version'
    )
    commands=(
        'pull:Pull and register a model directly'
        'setup:Bootstrap model family and speculative router'
        'admin:Autonomous OS self-healing & administration'
        'companion:Linux Cognitive Desktop Companion'
        'telemetry:Dynamic kernel telemetry & closed-loop PSI tuning'
        'tune:Dynamic tuning governor'
        'visual:Generative visual image synthesis'
        'audio:Generative music & acoustic atmosphere'
        'video:Generative short-form video synthesis'
        'decide:Syntrop decision engine'
        'audit:Security and system audit'
        'prompt:Sub-millisecond shell prompt hook'
        'talk:Real-time voice companion loop'
        'completions:Generate shell completion scripts'
    )
    shells=('bash' 'zsh' 'fish')
    effort_tiers=('none' 'low' 'med' 'high' 'max')
    families=('qwen' 'granite' 'gemma' 'phi')
    policies=('balanced' 'throughput' 'low-latency')

    _arguments -C \
        '(-h --help)'{-h,--help}'[Print help]' \
        '(-V --version)'{-V,--version}'[Print version]' \
        '(-e --effort)'{-e,--effort}'[Set reasoning effort tier]:tier:(none low med high max)' \
        '1:command:->cmd' \
        '*::args:->args'

    case $state in
        cmd)
            _describe -t namespaces 'namespaces' namespaces
            _describe -t commands 'commands' commands
            ;;
        args)
            case $words[1] in
                completions|completion) _values 'shell' $shells ;;
                setup) _arguments '--family[Model family]:family:(qwen granite gemma phi)' '--dry-run[Simulate setup]' ;;
                tune) _arguments '(-p --policy)'{-p,--policy}'[Tuning policy]:policy:(balanced throughput low-latency)' 'status[Status]' ;;
                admin) _values 'admin command' status remediate rollback audit lockout ;;
                companion) _values 'companion command' status ask execute listen stop talk ;;
                router) _values 'router command' ask setup models default test ;;
                system) _values 'system command' units triage version status ;;
                telemetry) _values 'telemetry command' status tune ;;
                visual|audio|video) _values 'multimedia command' generate ;;
            esac ;;
    esac
}
_syn "$@"
"#;

/// Fish completion script for `syn` and `syntrop`.
pub const FISH_COMPLETION: &str = r#"# fish completion for syn / syntrop
function __syn_no_subcommand
    for i in (commandline -opc)
        if contains -- $i router runtime store hardware context tools fleet system pull setup admin companion talk telemetry tune visual audio video decide audit prompt completions
            return 1
        end
    end
    return 0
end

complete -c syn -n "__syn_no_subcommand" -s h -l help -d "Print help"
complete -c syn -n "__syn_no_subcommand" -s V -l version -d "Print version"
complete -c syn -n "__syn_no_subcommand" -s e -l effort -r -f -a "none low med high max" -d "Set reasoning effort tier"

for ns in router runtime store hardware context tools fleet system pull setup admin companion talk telemetry tune visual audio video decide audit prompt completions
    complete -c syn -n "__syn_no_subcommand" -a $ns
end

complete -c syn -n "__fish_seen_subcommand_from completions" -a "bash zsh fish"
complete -c syn -n "__fish_seen_subcommand_from setup" -l family -r -f -a "qwen granite gemma phi"
complete -c syn -n "__fish_seen_subcommand_from setup" -l dry-run
complete -c syn -n "__fish_seen_subcommand_from tune" -s p -l policy -r -f -a "balanced throughput low-latency"
complete -c syn -n "__fish_seen_subcommand_from admin" -a "status remediate rollback audit lockout"
complete -c syn -n "__fish_seen_subcommand_from companion" -a "status ask execute listen stop talk"
complete -c syn -n "__fish_seen_subcommand_from router" -a "ask setup models default test"
complete -c syn -n "__fish_seen_subcommand_from system" -a "units triage version status"
complete -c syn -n "__fish_seen_subcommand_from telemetry" -a "status tune"
complete -c syn -n "__fish_seen_subcommand_from visual audio video" -a "generate"

complete -c syntrop -w syn
"#;

/// Generates completion script string for the specified shell.
pub fn generate_completion(shell: &str) -> Option<&'static str> {
    match shell {
        "bash" => Some(BASH_COMPLETION),
        "zsh" => Some(ZSH_COMPLETION),
        "fish" => Some(FISH_COMPLETION),
        _ => None,
    }
}

/// Dispatches `syn completions [shell]`.
pub fn handle_syn_completions(args: &[String]) -> anyhow::Result<()> {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        println!("usage: syn completions [bash|zsh|fish]");
        println!();
        println!("Generate shell completion scripts for syntrop (syn).");
        println!();
        println!("Supported shells:");
        println!("  bash    Bourne Again SHell");
        println!("  zsh     Z Shell");
        println!("  fish    Friendly Interactive SHell");
        return Ok(());
    }

    let shell = &args[0];
    if let Some(script) = generate_completion(shell) {
        print!("{script}");
        Ok(())
    } else {
        eprintln!("unsupported shell '{shell}'. supported: bash, zsh, fish");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_completion_all_supported() {
        assert!(generate_completion("bash").unwrap().contains("complete -F _syn"));
        assert!(generate_completion("zsh").unwrap().contains("#compdef syn"));
        assert!(generate_completion("fish").unwrap().contains("complete -c syn"));
        assert!(generate_completion("powershell").is_none());
    }

    #[test]
    fn test_handle_syn_completions_help() {
        assert!(handle_syn_completions(&[]).is_ok());
        assert!(handle_syn_completions(&["--help".to_string()]).is_ok());
    }
}
