import datetime
import os

import yaml

# Configuration for the Sphinx documentation builder.
# All configuration specific to your project should be done in this file.
#
# A complete list of built-in Sphinx configuration values:
# https://www.sphinx-doc.org/en/master/usage/configuration.html
#
# Our starter pack uses the custom Canonical Sphinx extension
# to keep all documentation based on it consistent and on brand:
# https://github.com/canonical/canonical-sphinx


#######################
# Project information #
#######################

# Project name

project = "Hardware API"
author = "Canonical Ltd."


# Sidebar documentation title; best kept reasonably short

html_title = project + " documentation"


# Copyright string; shown at the bottom of the page
#
# Now, the starter pack uses CC-BY-SA as the license
# and the current year as the copyright year.
#
# NOTE: For static works, it is common to provide the first publication year.
#       Another option is to provide both the first year of publication
#       and the current year, especially for docs that frequently change,
#       e.g. 2022–2023 (note the en-dash).
#
#       A way to check a repo's creation date is to get a classic GitHub token
#       with 'repo' permissions; see https://github.com/settings/tokens
#       Next, use 'curl' and 'jq' to extract the date from the API's output:
#
#       curl -H 'Authorization: token <TOKEN>' \
#         -H 'Accept: application/vnd.github.v3.raw' \
#         https://api.github.com/repos/canonical/<REPO> | jq '.created_at'

copyright = "%s GPL-3.0, %s" % (datetime.date.today().year, author)


# Documentation website URL
#
# NOTE: The Open Graph Protocol (OGP) enhances page display in a social graph
#       and is used by social media platforms; see https://ogp.me/

ogp_site_url = "https://canonical-hardware-api.readthedocs-hosted.com/"


# Preview name of the documentation website

ogp_site_name = project


# Preview image URL

ogp_image = "https://assets.ubuntu.com/v1/253da317-image-document-ubuntudocs.svg"


# Product favicon; shown in bookmarks, browser tabs, etc.

# html_favicon = '.sphinx/_static/favicon.png'


# Dictionary of values to pass into the Sphinx context for all pages:
# https://www.sphinx-doc.org/en/master/usage/configuration.html#confval-html_context

html_context = {
    # Product page URL; can be different from product docs URL
    "product_page": "hw.ubuntu.com",
    # Product tag image; the orange part of your logo, shown in the page header
    # 'product_tag': '_static/tag.png',
    # Your Discourse instance URL
    "discourse": "https://discourse.ubuntu.com",
    # Your Mattermost channel URL
    "mattermost": "https://chat.canonical.com/canonical/channels/hardware-api",
    # Your Matrix channel URL
    "matrix": "",
    # Your documentation GitHub repository URL
    "github_url": "https://github.com/canonical/hardware-api",
    # Docs branch in the repo; used in links for viewing the source files
    "repo_default_branch": "main",
    # Docs location in the repo; used in links for viewing the source files
    "repo_folder": "/docs/",
    # To enable or disable the Previous / Next buttons at the bottom of pages
    # Valid options: none, prev, next, both
    # "sequential_nav": "both",
    # To enable listing contributors on individual pages, set to True
    "display_contributors": False,
    # Required for feedback button
    "github_issues": "enabled",
}

# To enable the edit button on pages, uncomment and change the link to a
# public repository on GitHub or Launchpad. Any of the following link domains
# are accepted:
# - https://github.com/example-org/example"
# - https://launchpad.net/example
# - https://git.launchpad.net/example
#
html_theme_options = {
    "source_edit_link": "https://github.com/canonical/hardware-api",
}

# Project slug; see https://meta.discourse.org/t/what-is-category-slug/87897

slug = "hardware-api"

#######################
# Sitemap configuration: https://sphinx-sitemap.readthedocs.io/
#######################

# Base URL of RTD hosted project

html_baseurl = "https://canonical-hardware-api.readthedocs-hosted.com/"

# URL scheme. Add language and version scheme elements.
# When configured with RTD variables, check for RTD environment so manual runs succeed:

if "READTHEDOCS_VERSION" in os.environ:
    version = os.environ["READTHEDOCS_VERSION"]
    sitemap_url_scheme = "{version}{link}"
else:
    sitemap_url_scheme = "MANUAL/{link}"

# Include `lastmod` dates in the sitemap:

sitemap_show_lastmod = True

#######################
# Template and asset locations
#######################

html_static_path = [".sphinx/_static"]
# templates_path = ["_templates"]

# Extra files to copy to the root of the documentation
# https://www.sphinx-doc.org/en/master/usage/configuration.html#confval-html_extra_path
html_extra_path = ["../server/schemas/openapi.yaml"]

#############
# Redirects #
#############

# To set up redirects: https://documatt.gitlab.io/sphinx-reredirects/usage.html
# For example: 'explanation/old-name.html': '../how-to/prettify.html',

# To set up redirects in the Read the Docs project dashboard:
# https://docs.readthedocs.io/en/stable/guides/redirects.html

# NOTE: If undefined, set to None, or empty,
#       the sphinx_reredirects extension will be disabled.

redirects = {}


###########################
# Link checker exceptions #
###########################

# A regex list of URLs that are ignored by 'make linkcheck'

linkcheck_ignore = [
    "http://127.0.0.1:8000",
    "https://github.com/canonical/hardware-api/*",
    "https://canonical.github.io/hardware-api/",
]


# A regex list of URLs where anchors are ignored by 'make linkcheck'

linkcheck_anchors_ignore_for_url = [r"https://github\.com/.*"]

# give linkcheck multiple tries on failure
# linkcheck_timeout = 30
linkcheck_retries = 3

########################
# Configuration extras #
########################

# Custom MyST syntax extensions; see
# https://myst-parser.readthedocs.io/en/latest/syntax/optional.html
#
# NOTE: By default, the following MyST extensions are enabled:
#       substitution, deflist, linkify

# myst_enable_extensions = set()


# Custom Sphinx extensions; see
# https://www.sphinx-doc.org/en/master/usage/extensions/index.html

# NOTE: The canonical_sphinx extension is required for the starter pack.
#       It automatically enables the following extensions:
#       - custom-rst-roles
#       - myst_parser
#       - notfound.extension
#       - related-links
#       - sphinx_copybutton
#       - sphinx_design
#       - sphinx_reredirects
#       - sphinx_tabs.tabs
#       - sphinxcontrib.jquery
#       - sphinxext.opengraph
#       - terminal-output
#       - youtube-links

extensions = [
    "canonical_sphinx",
    "sphinxcontrib.cairosvgconverter",
    "sphinx_last_updated_by_git",
    "sphinx.ext.intersphinx",
    "sphinx_sitemap",
]

# Excludes files or directories from processing

exclude_patterns = [
    "doc-cheat-sheet*",
]

# Adds custom CSS files, located under 'html_static_path'

# html_css_files = []


# Adds custom JavaScript files, located under 'html_static_path'

# html_js_files = []


# Specifies a reST snippet to be appended to each .rst file

rst_epilog = """
.. include:: /reuse/links.txt
.. include:: /reuse/substitutions.txt
"""

# Feedback button at the top; enabled by default

# disable_feedback_button = True


# Your manpage URL
#
# NOTE: If set, adding ':manpage:' to an .rst file
#       adds a link to the corresponding man section at the bottom of the page.

manpages_url = "https://manpages.ubuntu.com/manpages/resolute/en/man{section}/{page}.{section}.html"


# Specifies a reST snippet to be prepended to each .rst file
# This defines a :center: role that centers table cell content.
# This defines a :h2: role that styles content for use with PDF generation.

rst_prolog = """
.. role:: center
   :class: align-center
.. role:: h2
    :class: hclass2
.. role:: woke-ignore
    :class: woke-ignore
.. role:: vale-ignore
    :class: vale-ignore
"""

# Workaround for https://github.com/canonical/canonical-sphinx/issues/34

if "discourse_prefix" not in html_context and "discourse" in html_context:
    html_context["discourse_prefix"] = html_context["discourse"] + "/t/"

# Workaround for substitutions.yaml

if os.path.exists("./reuse/substitutions.yaml"):
    with open("./reuse/substitutions.yaml", "r") as fd:
        myst_substitutions = yaml.safe_load(fd.read())

# Add configuration for intersphinx mapping

intersphinx_mapping = {
    "starter-pack": (
        "https://canonical-example-product-documentation.readthedocs-hosted.com/en/latest",
        None,
    )
}
