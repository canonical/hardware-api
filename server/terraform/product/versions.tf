terraform {
  required_version = ">= 1.7.2"
  required_providers {
    juju = {
      source  = "juju/juju"
      version = "~> 0.23.0"
    }
  }
}
