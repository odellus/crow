"""Config loader for task pipeline."""

import os
import re
from pathlib import Path
from typing import Any

import yaml


def load_config(config_path: Path) -> dict[str, Any]:
    """
    Load configuration from YAML file.
    
    Args:
        config_path: Path to YAML config file
        
    Returns:
        Configuration dictionary with all settings
        
    Raises:
        FileNotFoundError: If config file doesn't exist
        ValueError: If required fields are missing
    """
    if not config_path.exists():
        raise FileNotFoundError(f"Config file not found: {config_path}")
    
    with open(config_path) as f:
        config = yaml.safe_load(f)
    
    # Expand environment variables
    config = _expand_env_vars(config)
    
    # Validate required fields
    _validate_config(config)
    
    return config


def _expand_env_vars(obj: Any) -> Any:
    """
    Recursively expand ${ENV_VAR} syntax in config values.
    
    Args:
        obj: Any object (dict, list, str, etc.)
        
    Returns:
        Object with env vars expanded
    """
    if isinstance(obj, dict):
        return {k: _expand_env_vars(v) for k, v in obj.items()}
    elif isinstance(obj, list):
        return [_expand_env_vars(item) for item in obj]
    elif isinstance(obj, str):
        # Expand ${VAR} or $VAR syntax
        def replace_env_var(match):
            var_name = match.group(1) or match.group(2)
            return os.getenv(var_name, match.group(0))
        
        # Match ${VAR} or $VAR
        pattern = r'\$\{([^}]+)\}|\$([a-zA-Z_][a-zA-Z0-9_]*)'
        return re.sub(pattern, replace_env_var, obj)
    else:
        return obj


def _validate_config(config: dict[str, Any]) -> None:
    """
    Validate that required config fields exist.
    
    Args:
        config: Configuration dictionary
        
    Raises:
        ValueError: If required fields are missing
    """
    required_fields = ["prompts"]
    
    for field in required_fields:
        if field not in config:
            raise ValueError(f"Missing required config field: {field}")
    
    # Validate prompts section
    if "prompts" in config:
        required_prompts = ["planning", "implementation", "critic"]
        for prompt_type in required_prompts:
            if prompt_type not in config["prompts"]:
                raise ValueError(f"Missing required prompt config: prompts.{prompt_type}")


def merge_cli_args(config: dict[str, Any], cli_args: dict[str, Any]) -> dict[str, Any]:
    """
    Merge CLI arguments into config (CLI args take precedence).
    
    Args:
        config: Base configuration from YAML
        cli_args: CLI arguments to override
        
    Returns:
        Merged configuration
    """
    result = config.copy()
    
    # Override prompts if provided
    if "planning_prompt" in cli_args:
        result.setdefault("prompts", {})["planning"] = cli_args["planning_prompt"]
    if "implementation_prompt" in cli_args:
        result.setdefault("prompts", {})["implementation"] = cli_args["implementation_prompt"]
    if "critic_prompt" in cli_args:
        result.setdefault("prompts", {})["critic"] = cli_args["critic_prompt"]
    if "documentation_prompt" in cli_args:
        result.setdefault("prompts", {})["documentation"] = cli_args["documentation_prompt"]
    
    # Override defaults if provided
    if "quality_threshold" in cli_args:
        result.setdefault("defaults", {})["quality_threshold"] = cli_args["quality_threshold"]
    if "max_iterations" in cli_args:
        result.setdefault("defaults", {})["max_iterations"] = cli_args["max_iterations"]
    
    # Override paths if provided
    if "workspace_dir" in cli_args:
        result.setdefault("pipeline", {})["workspace_dir"] = cli_args["workspace_dir"]
    if "tasks_dir" in cli_args:
        result.setdefault("pipeline", {})["tasks_dir"] = cli_args["tasks_dir"]
    
    return result
