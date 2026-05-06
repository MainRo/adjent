# Adjent configuration

The local configuration is stored on the filesystem. The base directory of this configuration is retrieved with these different ways, by order of priority:

- First, from the environment variable ADJENT_HOME if it exists.
- Otherwise from the subdirectory .adjent of the environment variable HOME

The local state is in the "state" subdirectory of the base state.