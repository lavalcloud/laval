# Laval Helm Chart
#
# This Helm chart deploys the Laval application, including database, manager, and frontend services.
#
# ## Installation
#
# To install the chart with the release name `my-release`:
#
# ```bash
# helm install my-release ./laval
# ```
#
# ## Configuration
#
# The following table lists the configurable parameters of the Laval chart and their default values.
#
# | Parameter | Description | Default |
# |-----------|-------------|---------|
# | `db.user` | Database username | `laval` |
# | `db.password` | Database password | `laval` |
# | `db.database` | Database name | `laval` |
# | `db.storage` | Database storage size | `1Gi` |
# | `manager.image` | Manager image | `laval-manager:latest` |
# | `manager.replicas` | Number of manager replicas | `1` |
# | `manager.logLevel` | Log level for manager | `info` |
# | `frontend.image` | Frontend image | `laval-frontend:latest` |
# | `frontend.replicas` | Number of frontend replicas | `1` |
# | `domain` | Domain for Gateway | `example.com` |
#
# Specify each parameter using the `--set key=value[,key=value]` argument to `helm install`.