#/bin/bash

out="AUTHORS"

cat <<EOF > $out
# The Authors of Vendor
#
# This is the list of authors of the project for copyright purposes.
# When you contribute to the project you automatically become an author.
#
# Do not edit manually, this file is auto-generated.

EOF

git shortlog -sec | awk -F '\t' '{ print $2 }' >> $out
