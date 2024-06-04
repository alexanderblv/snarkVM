sed -i '' 's@resource_class: xlarge@resource_class: << parameters.xlarge >>' config.yml
sed -i '' 's@resource_class: 2xlarge@resource_class: << parameters.2xlarge >>@g' config.yml