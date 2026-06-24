variable "model" {
  type        = string
  description = "Reference to an existing model resource or data source for the model to deploy to"
}

variable "owner" {
  type        = string
  default     = null
  description = "Reference to the owner of the model to deploy to"
}

variable "hardware_api" {
  type = object({
    app_name    = optional(string, "api")
    channel     = optional(string, "latest/edge")
    config      = optional(map(string), {})
    constraints = optional(string, "arch=amd64")
    revision    = optional(number)
    base        = optional(string, "ubuntu@22.04")
    units       = optional(number, 1)
  })
  default = {}
}

variable "ingress_configurator" {
  type = object({
    app_name = optional(string, "ingress-configurator")
    channel  = optional(string, "latest/edge")
    config   = optional(map(string), {})
    revision = optional(number)
    units    = optional(number, 1)
  })
  default = {}
}

variable "haproxy_offer" {
  type        = string
  nullable    = true
  default     = null
  description = "URL to the haproxy offer for ingress relation. If null, the integration is not created."
}

