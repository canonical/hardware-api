variable "model" {
  type        = string
  nullable    = true
  default     = null
  description = "Reference to an existing model resource or data source for the model to deploy to"
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
}

variable "nginx_ingress_integrator" {
  type = object({
    app_name = optional(string, "ingress")
    channel  = optional(string, "latest/stable")
    config   = optional(map(string), {})
    revision = optional(number)
    units    = optional(number, 1)
  })
}
