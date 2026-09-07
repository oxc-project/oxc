# externals/aws-cloudformation-templates/Solutions/OperatingSystems/ubuntu20.04_cfn-hup.yaml

> block scalar trailing whitespace is part of the value. See crates/oxc_formatter_yaml/DIVERGENCES.md#block-scalar-trailing-whitespace

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -242,9 +242,9 @@
                 [cfn-auto-reloader-hook]
                 triggers=post.update
                 path=Resources.WebServerInstance.Metadata.AWS::CloudFormation::Init
                 action=/opt/aws/bin/cfn-init -v --stack ${AWS::StackName} --resource WebServerInstance --configsets InstallAndRun --region ${AWS::Region}
-                runas=root
+                runas=root              
               mode: "000400"
               owner: root
               group: root
             /lib/systemd/system/cfn-hup.service:

`````

### Actual (oxfmt)

`````yaml
AWSTemplateFormatVersion: "2010-09-09"

Description: Installing Cloudformation helper scripts in Ubuntu 20.04 LTS

Parameters:
  KeyName:
    Description: Name of an existing EC2 KeyPair to enable SSH access to the instance
    Type: AWS::EC2::KeyPair::KeyName

  InstanceType:
    Description: WebServer EC2 instance type
    Type: String
    AllowedValues:
      - t1.micro
      - t2.nano
      - t2.micro
      - t2.small
      - t2.medium
      - t2.large
      - m1.small
      - m1.medium
      - m1.large
      - m1.xlarge
      - m2.xlarge
      - m2.2xlarge
      - m2.4xlarge
      - m3.medium
      - m3.large
      - m3.xlarge
      - m3.2xlarge
      - m4.large
      - m4.xlarge
      - m4.2xlarge
      - m4.4xlarge
      - m4.10xlarge
      - c1.medium
      - c1.xlarge
      - c3.large
      - c3.xlarge
      - c3.2xlarge
      - c3.4xlarge
      - c3.8xlarge
      - c4.large
      - c4.xlarge
      - c4.2xlarge
      - c4.4xlarge
      - c4.8xlarge
      - r3.large
      - r3.xlarge
      - r3.2xlarge
      - r3.4xlarge
      - r3.8xlarge
      - i2.xlarge
      - i2.2xlarge
      - i2.4xlarge
      - i2.8xlarge
      - d2.xlarge
      - d2.2xlarge
      - d2.4xlarge
      - d2.8xlarge
      - hs1.8xlarge
      - cr1.8xlarge
      - cc2.8xlarge
    Default: t2.small
    ConstraintDescription: must be a valid EC2 instance type.

  SSHLocation:
    Description: The IP address range that can be used to SSH to the EC2 instances
    Type: String
    Default: 0.0.0.0/0
    MinLength: "9"
    MaxLength: "18"
    AllowedPattern: (\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})/(\d{1,2})
    ConstraintDescription: must be a valid IP CIDR range of the form x.x.x.x/x.

  SubnetId:
    Type: AWS::EC2::Subnet::Id

Mappings:
  AWSInstanceType2Arch:
    t1.micro:
      Arch: HVM64
    t2.nano:
      Arch: HVM64
    t2.micro:
      Arch: HVM64
    t2.small:
      Arch: HVM64
    t2.medium:
      Arch: HVM64
    t2.large:
      Arch: HVM64
    m1.small:
      Arch: HVM64
    m1.medium:
      Arch: HVM64
    m1.large:
      Arch: HVM64
    m1.xlarge:
      Arch: HVM64
    m2.xlarge:
      Arch: HVM64
    m2.2xlarge:
      Arch: HVM64
    m2.4xlarge:
      Arch: HVM64
    m3.medium:
      Arch: HVM64
    m3.large:
      Arch: HVM64
    m3.xlarge:
      Arch: HVM64
    m3.2xlarge:
      Arch: HVM64
    m4.large:
      Arch: HVM64
    m4.xlarge:
      Arch: HVM64
    m4.2xlarge:
      Arch: HVM64
    m4.4xlarge:
      Arch: HVM64
    m4.10xlarge:
      Arch: HVM64
    c1.medium:
      Arch: HVM64
    c1.xlarge:
      Arch: HVM64
    c3.large:
      Arch: HVM64
    c3.xlarge:
      Arch: HVM64
    c3.2xlarge:
      Arch: HVM64
    c3.4xlarge:
      Arch: HVM64
    c3.8xlarge:
      Arch: HVM64
    c4.large:
      Arch: HVM64
    c4.xlarge:
      Arch: HVM64
    c4.2xlarge:
      Arch: HVM64
    c4.4xlarge:
      Arch: HVM64
    c4.8xlarge:
      Arch: HVM64
    r3.large:
      Arch: HVM64
    r3.xlarge:
      Arch: HVM64
    r3.2xlarge:
      Arch: HVM64
    r3.4xlarge:
      Arch: HVM64
    r3.8xlarge:
      Arch: HVM64
    i2.xlarge:
      Arch: HVM64
    i2.2xlarge:
      Arch: HVM64
    i2.4xlarge:
      Arch: HVM64
    i2.8xlarge:
      Arch: HVM64
    d2.xlarge:
      Arch: HVM64
    d2.2xlarge:
      Arch: HVM64
    d2.4xlarge:
      Arch: HVM64
    d2.8xlarge:
      Arch: HVM64
    hi1.4xlarge:
      Arch: HVM64
    hs1.8xlarge:
      Arch: HVM64
    cr1.8xlarge:
      Arch: HVM64
    cc2.8xlarge:
      Arch: HVM64

  AWSRegionArch2AMI:
    us-east-1:
      HVM64: ami-0149b2da6ceec4bb0
    us-west-2:
      HVM64: ami-0c09c7eb16d3e8e70
    us-west-1:
      HVM64: ami-03f6d497fceb40069
    eu-west-1:
      HVM64: ami-0fd8802f94ed1c969
    eu-west-2:
      HVM64: ami-04842bc62789b682e
    eu-west-3:
      HVM64: ami-064736ff8301af3ee
    eu-central-1:
      HVM64: ami-06148e0e81e5187c8
    ap-northeast-1:
      HVM64: ami-09b18720cb71042df
    ap-northeast-2:
      HVM64: ami-07d16c043aa8e5153
    ap-northeast-3:
      HVM64: ami-09d2f3a31110c6ad4
    ap-southeast-1:
      HVM64: ami-00e912d13fbb4f225
    ap-southeast-2:
      HVM64: ami-055166f8a8041fbf1
    ap-south-1:
      HVM64: ami-024c319d5d14b463e
    us-east-2:
      HVM64: ami-0d5bf08bc8017c83b
    ca-central-1:
      HVM64: ami-043a72cf696697251
    sa-east-1:
      HVM64: ami-00742e66d44c13cd9

Resources:
  EC2Instance:
    CreationPolicy:
      ResourceSignal:
        Timeout: PT10M
        Count: "1"
    Type: AWS::EC2::Instance
    Metadata:
      AWS::CloudFormation::Init:
        configSets:
          full_install:
            - install_and_enable_cfn_hup
        install_and_enable_cfn_hup:
          files:
            /etc/cfn/cfn-hup.conf:
              content: !Sub |
                [main]
                stack=${AWS::StackId}
                region=${AWS::Region}
              mode: "000400"
              owner: root
              group: root
            /etc/cfn/hooks.d/cfn-auto-reloader.conf:
              content: !Sub |
                [cfn-auto-reloader-hook]
                triggers=post.update
                path=Resources.WebServerInstance.Metadata.AWS::CloudFormation::Init
                action=/opt/aws/bin/cfn-init -v --stack ${AWS::StackName} --resource WebServerInstance --configsets InstallAndRun --region ${AWS::Region}
                runas=root              
              mode: "000400"
              owner: root
              group: root
            /lib/systemd/system/cfn-hup.service:
              content: |
                [Unit]
                Description=cfn-hup daemon
                [Service]
                Type=simple
                ExecStart=/usr/local/bin/cfn-hup
                Restart=always
                [Install]
                WantedBy=multi-user.target
          commands:
            01enable_cfn_hup:
              command: systemctl enable cfn-hup.service
            02start_cfn_hup:
              command: systemctl start cfn-hup.service
    Properties:
      InstanceType: !Ref InstanceType
      SubnetId: !Ref SubnetId
      SecurityGroupIds:
        - !GetAtt InstanceSecurityGroup.GroupId
      KeyName: !Ref KeyName
      ImageId: !FindInMap
        - AWSRegionArch2AMI
        - !Ref AWS::Region
        - !FindInMap
          - AWSInstanceType2Arch
          - !Ref InstanceType
          - Arch
      UserData: !Base64
        Fn::Sub: |
          #!/bin/bash -xe
          sudo apt-get update -y
          sudo apt-get -y install python3-pip
          mkdir -p /opt/aws/
          sudo pip3 install https://s3.amazonaws.com/cloudformation-examples/aws-cfn-bootstrap-py3-latest.tar.gz
          sudo ln -s /usr/local/init/ubuntu/cfn-hup /etc/init.d/cfn-hup
          /usr/local/bin/cfn-init -v --stack ${AWS::StackName} --resource EC2Instance --configsets full_install --region ${AWS::Region}
          /usr/local/bin/cfn-signal -e $?  --stack ${AWS::StackName} --resource EC2Instance --region ${AWS::Region}

  InstanceSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Enable SSH access via port 22
      SecurityGroupIngress:
        - IpProtocol: tcp
          FromPort: "22"
          ToPort: "22"
          CidrIp: !Ref SSHLocation

`````

### Expected (prettier)

`````yaml
AWSTemplateFormatVersion: "2010-09-09"

Description: Installing Cloudformation helper scripts in Ubuntu 20.04 LTS

Parameters:
  KeyName:
    Description: Name of an existing EC2 KeyPair to enable SSH access to the instance
    Type: AWS::EC2::KeyPair::KeyName

  InstanceType:
    Description: WebServer EC2 instance type
    Type: String
    AllowedValues:
      - t1.micro
      - t2.nano
      - t2.micro
      - t2.small
      - t2.medium
      - t2.large
      - m1.small
      - m1.medium
      - m1.large
      - m1.xlarge
      - m2.xlarge
      - m2.2xlarge
      - m2.4xlarge
      - m3.medium
      - m3.large
      - m3.xlarge
      - m3.2xlarge
      - m4.large
      - m4.xlarge
      - m4.2xlarge
      - m4.4xlarge
      - m4.10xlarge
      - c1.medium
      - c1.xlarge
      - c3.large
      - c3.xlarge
      - c3.2xlarge
      - c3.4xlarge
      - c3.8xlarge
      - c4.large
      - c4.xlarge
      - c4.2xlarge
      - c4.4xlarge
      - c4.8xlarge
      - r3.large
      - r3.xlarge
      - r3.2xlarge
      - r3.4xlarge
      - r3.8xlarge
      - i2.xlarge
      - i2.2xlarge
      - i2.4xlarge
      - i2.8xlarge
      - d2.xlarge
      - d2.2xlarge
      - d2.4xlarge
      - d2.8xlarge
      - hs1.8xlarge
      - cr1.8xlarge
      - cc2.8xlarge
    Default: t2.small
    ConstraintDescription: must be a valid EC2 instance type.

  SSHLocation:
    Description: The IP address range that can be used to SSH to the EC2 instances
    Type: String
    Default: 0.0.0.0/0
    MinLength: "9"
    MaxLength: "18"
    AllowedPattern: (\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})/(\d{1,2})
    ConstraintDescription: must be a valid IP CIDR range of the form x.x.x.x/x.

  SubnetId:
    Type: AWS::EC2::Subnet::Id

Mappings:
  AWSInstanceType2Arch:
    t1.micro:
      Arch: HVM64
    t2.nano:
      Arch: HVM64
    t2.micro:
      Arch: HVM64
    t2.small:
      Arch: HVM64
    t2.medium:
      Arch: HVM64
    t2.large:
      Arch: HVM64
    m1.small:
      Arch: HVM64
    m1.medium:
      Arch: HVM64
    m1.large:
      Arch: HVM64
    m1.xlarge:
      Arch: HVM64
    m2.xlarge:
      Arch: HVM64
    m2.2xlarge:
      Arch: HVM64
    m2.4xlarge:
      Arch: HVM64
    m3.medium:
      Arch: HVM64
    m3.large:
      Arch: HVM64
    m3.xlarge:
      Arch: HVM64
    m3.2xlarge:
      Arch: HVM64
    m4.large:
      Arch: HVM64
    m4.xlarge:
      Arch: HVM64
    m4.2xlarge:
      Arch: HVM64
    m4.4xlarge:
      Arch: HVM64
    m4.10xlarge:
      Arch: HVM64
    c1.medium:
      Arch: HVM64
    c1.xlarge:
      Arch: HVM64
    c3.large:
      Arch: HVM64
    c3.xlarge:
      Arch: HVM64
    c3.2xlarge:
      Arch: HVM64
    c3.4xlarge:
      Arch: HVM64
    c3.8xlarge:
      Arch: HVM64
    c4.large:
      Arch: HVM64
    c4.xlarge:
      Arch: HVM64
    c4.2xlarge:
      Arch: HVM64
    c4.4xlarge:
      Arch: HVM64
    c4.8xlarge:
      Arch: HVM64
    r3.large:
      Arch: HVM64
    r3.xlarge:
      Arch: HVM64
    r3.2xlarge:
      Arch: HVM64
    r3.4xlarge:
      Arch: HVM64
    r3.8xlarge:
      Arch: HVM64
    i2.xlarge:
      Arch: HVM64
    i2.2xlarge:
      Arch: HVM64
    i2.4xlarge:
      Arch: HVM64
    i2.8xlarge:
      Arch: HVM64
    d2.xlarge:
      Arch: HVM64
    d2.2xlarge:
      Arch: HVM64
    d2.4xlarge:
      Arch: HVM64
    d2.8xlarge:
      Arch: HVM64
    hi1.4xlarge:
      Arch: HVM64
    hs1.8xlarge:
      Arch: HVM64
    cr1.8xlarge:
      Arch: HVM64
    cc2.8xlarge:
      Arch: HVM64

  AWSRegionArch2AMI:
    us-east-1:
      HVM64: ami-0149b2da6ceec4bb0
    us-west-2:
      HVM64: ami-0c09c7eb16d3e8e70
    us-west-1:
      HVM64: ami-03f6d497fceb40069
    eu-west-1:
      HVM64: ami-0fd8802f94ed1c969
    eu-west-2:
      HVM64: ami-04842bc62789b682e
    eu-west-3:
      HVM64: ami-064736ff8301af3ee
    eu-central-1:
      HVM64: ami-06148e0e81e5187c8
    ap-northeast-1:
      HVM64: ami-09b18720cb71042df
    ap-northeast-2:
      HVM64: ami-07d16c043aa8e5153
    ap-northeast-3:
      HVM64: ami-09d2f3a31110c6ad4
    ap-southeast-1:
      HVM64: ami-00e912d13fbb4f225
    ap-southeast-2:
      HVM64: ami-055166f8a8041fbf1
    ap-south-1:
      HVM64: ami-024c319d5d14b463e
    us-east-2:
      HVM64: ami-0d5bf08bc8017c83b
    ca-central-1:
      HVM64: ami-043a72cf696697251
    sa-east-1:
      HVM64: ami-00742e66d44c13cd9

Resources:
  EC2Instance:
    CreationPolicy:
      ResourceSignal:
        Timeout: PT10M
        Count: "1"
    Type: AWS::EC2::Instance
    Metadata:
      AWS::CloudFormation::Init:
        configSets:
          full_install:
            - install_and_enable_cfn_hup
        install_and_enable_cfn_hup:
          files:
            /etc/cfn/cfn-hup.conf:
              content: !Sub |
                [main]
                stack=${AWS::StackId}
                region=${AWS::Region}
              mode: "000400"
              owner: root
              group: root
            /etc/cfn/hooks.d/cfn-auto-reloader.conf:
              content: !Sub |
                [cfn-auto-reloader-hook]
                triggers=post.update
                path=Resources.WebServerInstance.Metadata.AWS::CloudFormation::Init
                action=/opt/aws/bin/cfn-init -v --stack ${AWS::StackName} --resource WebServerInstance --configsets InstallAndRun --region ${AWS::Region}
                runas=root
              mode: "000400"
              owner: root
              group: root
            /lib/systemd/system/cfn-hup.service:
              content: |
                [Unit]
                Description=cfn-hup daemon
                [Service]
                Type=simple
                ExecStart=/usr/local/bin/cfn-hup
                Restart=always
                [Install]
                WantedBy=multi-user.target
          commands:
            01enable_cfn_hup:
              command: systemctl enable cfn-hup.service
            02start_cfn_hup:
              command: systemctl start cfn-hup.service
    Properties:
      InstanceType: !Ref InstanceType
      SubnetId: !Ref SubnetId
      SecurityGroupIds:
        - !GetAtt InstanceSecurityGroup.GroupId
      KeyName: !Ref KeyName
      ImageId: !FindInMap
        - AWSRegionArch2AMI
        - !Ref AWS::Region
        - !FindInMap
          - AWSInstanceType2Arch
          - !Ref InstanceType
          - Arch
      UserData: !Base64
        Fn::Sub: |
          #!/bin/bash -xe
          sudo apt-get update -y
          sudo apt-get -y install python3-pip
          mkdir -p /opt/aws/
          sudo pip3 install https://s3.amazonaws.com/cloudformation-examples/aws-cfn-bootstrap-py3-latest.tar.gz
          sudo ln -s /usr/local/init/ubuntu/cfn-hup /etc/init.d/cfn-hup
          /usr/local/bin/cfn-init -v --stack ${AWS::StackName} --resource EC2Instance --configsets full_install --region ${AWS::Region}
          /usr/local/bin/cfn-signal -e $?  --stack ${AWS::StackName} --resource EC2Instance --region ${AWS::Region}

  InstanceSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Enable SSH access via port 22
      SecurityGroupIngress:
        - IpProtocol: tcp
          FromPort: "22"
          ToPort: "22"
          CidrIp: !Ref SSHLocation

`````

## Option 2

`````json
{"printWidth":100,"tabWidth":4,"proseWrap":"always"}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -242,9 +242,9 @@
                                 [cfn-auto-reloader-hook]
                                 triggers=post.update
                                 path=Resources.WebServerInstance.Metadata.AWS::CloudFormation::Init
                                 action=/opt/aws/bin/cfn-init -v --stack ${AWS::StackName} --resource WebServerInstance --configsets InstallAndRun --region ${AWS::Region}
-                                runas=root
+                                runas=root              
                             mode: "000400"
                             owner: root
                             group: root
                         /lib/systemd/system/cfn-hup.service:

`````

### Actual (oxfmt)

`````yaml
AWSTemplateFormatVersion: "2010-09-09"

Description: Installing Cloudformation helper scripts in Ubuntu 20.04 LTS

Parameters:
    KeyName:
        Description: Name of an existing EC2 KeyPair to enable SSH access to the instance
        Type: AWS::EC2::KeyPair::KeyName

    InstanceType:
        Description: WebServer EC2 instance type
        Type: String
        AllowedValues:
            - t1.micro
            - t2.nano
            - t2.micro
            - t2.small
            - t2.medium
            - t2.large
            - m1.small
            - m1.medium
            - m1.large
            - m1.xlarge
            - m2.xlarge
            - m2.2xlarge
            - m2.4xlarge
            - m3.medium
            - m3.large
            - m3.xlarge
            - m3.2xlarge
            - m4.large
            - m4.xlarge
            - m4.2xlarge
            - m4.4xlarge
            - m4.10xlarge
            - c1.medium
            - c1.xlarge
            - c3.large
            - c3.xlarge
            - c3.2xlarge
            - c3.4xlarge
            - c3.8xlarge
            - c4.large
            - c4.xlarge
            - c4.2xlarge
            - c4.4xlarge
            - c4.8xlarge
            - r3.large
            - r3.xlarge
            - r3.2xlarge
            - r3.4xlarge
            - r3.8xlarge
            - i2.xlarge
            - i2.2xlarge
            - i2.4xlarge
            - i2.8xlarge
            - d2.xlarge
            - d2.2xlarge
            - d2.4xlarge
            - d2.8xlarge
            - hs1.8xlarge
            - cr1.8xlarge
            - cc2.8xlarge
        Default: t2.small
        ConstraintDescription: must be a valid EC2 instance type.

    SSHLocation:
        Description: The IP address range that can be used to SSH to the EC2 instances
        Type: String
        Default: 0.0.0.0/0
        MinLength: "9"
        MaxLength: "18"
        AllowedPattern: (\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})/(\d{1,2})
        ConstraintDescription: must be a valid IP CIDR range of the form x.x.x.x/x.

    SubnetId:
        Type: AWS::EC2::Subnet::Id

Mappings:
    AWSInstanceType2Arch:
        t1.micro:
            Arch: HVM64
        t2.nano:
            Arch: HVM64
        t2.micro:
            Arch: HVM64
        t2.small:
            Arch: HVM64
        t2.medium:
            Arch: HVM64
        t2.large:
            Arch: HVM64
        m1.small:
            Arch: HVM64
        m1.medium:
            Arch: HVM64
        m1.large:
            Arch: HVM64
        m1.xlarge:
            Arch: HVM64
        m2.xlarge:
            Arch: HVM64
        m2.2xlarge:
            Arch: HVM64
        m2.4xlarge:
            Arch: HVM64
        m3.medium:
            Arch: HVM64
        m3.large:
            Arch: HVM64
        m3.xlarge:
            Arch: HVM64
        m3.2xlarge:
            Arch: HVM64
        m4.large:
            Arch: HVM64
        m4.xlarge:
            Arch: HVM64
        m4.2xlarge:
            Arch: HVM64
        m4.4xlarge:
            Arch: HVM64
        m4.10xlarge:
            Arch: HVM64
        c1.medium:
            Arch: HVM64
        c1.xlarge:
            Arch: HVM64
        c3.large:
            Arch: HVM64
        c3.xlarge:
            Arch: HVM64
        c3.2xlarge:
            Arch: HVM64
        c3.4xlarge:
            Arch: HVM64
        c3.8xlarge:
            Arch: HVM64
        c4.large:
            Arch: HVM64
        c4.xlarge:
            Arch: HVM64
        c4.2xlarge:
            Arch: HVM64
        c4.4xlarge:
            Arch: HVM64
        c4.8xlarge:
            Arch: HVM64
        r3.large:
            Arch: HVM64
        r3.xlarge:
            Arch: HVM64
        r3.2xlarge:
            Arch: HVM64
        r3.4xlarge:
            Arch: HVM64
        r3.8xlarge:
            Arch: HVM64
        i2.xlarge:
            Arch: HVM64
        i2.2xlarge:
            Arch: HVM64
        i2.4xlarge:
            Arch: HVM64
        i2.8xlarge:
            Arch: HVM64
        d2.xlarge:
            Arch: HVM64
        d2.2xlarge:
            Arch: HVM64
        d2.4xlarge:
            Arch: HVM64
        d2.8xlarge:
            Arch: HVM64
        hi1.4xlarge:
            Arch: HVM64
        hs1.8xlarge:
            Arch: HVM64
        cr1.8xlarge:
            Arch: HVM64
        cc2.8xlarge:
            Arch: HVM64

    AWSRegionArch2AMI:
        us-east-1:
            HVM64: ami-0149b2da6ceec4bb0
        us-west-2:
            HVM64: ami-0c09c7eb16d3e8e70
        us-west-1:
            HVM64: ami-03f6d497fceb40069
        eu-west-1:
            HVM64: ami-0fd8802f94ed1c969
        eu-west-2:
            HVM64: ami-04842bc62789b682e
        eu-west-3:
            HVM64: ami-064736ff8301af3ee
        eu-central-1:
            HVM64: ami-06148e0e81e5187c8
        ap-northeast-1:
            HVM64: ami-09b18720cb71042df
        ap-northeast-2:
            HVM64: ami-07d16c043aa8e5153
        ap-northeast-3:
            HVM64: ami-09d2f3a31110c6ad4
        ap-southeast-1:
            HVM64: ami-00e912d13fbb4f225
        ap-southeast-2:
            HVM64: ami-055166f8a8041fbf1
        ap-south-1:
            HVM64: ami-024c319d5d14b463e
        us-east-2:
            HVM64: ami-0d5bf08bc8017c83b
        ca-central-1:
            HVM64: ami-043a72cf696697251
        sa-east-1:
            HVM64: ami-00742e66d44c13cd9

Resources:
    EC2Instance:
        CreationPolicy:
            ResourceSignal:
                Timeout: PT10M
                Count: "1"
        Type: AWS::EC2::Instance
        Metadata:
            AWS::CloudFormation::Init:
                configSets:
                    full_install:
                        - install_and_enable_cfn_hup
                install_and_enable_cfn_hup:
                    files:
                        /etc/cfn/cfn-hup.conf:
                            content: !Sub |
                                [main]
                                stack=${AWS::StackId}
                                region=${AWS::Region}
                            mode: "000400"
                            owner: root
                            group: root
                        /etc/cfn/hooks.d/cfn-auto-reloader.conf:
                            content: !Sub |
                                [cfn-auto-reloader-hook]
                                triggers=post.update
                                path=Resources.WebServerInstance.Metadata.AWS::CloudFormation::Init
                                action=/opt/aws/bin/cfn-init -v --stack ${AWS::StackName} --resource WebServerInstance --configsets InstallAndRun --region ${AWS::Region}
                                runas=root              
                            mode: "000400"
                            owner: root
                            group: root
                        /lib/systemd/system/cfn-hup.service:
                            content: |
                                [Unit]
                                Description=cfn-hup daemon
                                [Service]
                                Type=simple
                                ExecStart=/usr/local/bin/cfn-hup
                                Restart=always
                                [Install]
                                WantedBy=multi-user.target
                    commands:
                        01enable_cfn_hup:
                            command: systemctl enable cfn-hup.service
                        02start_cfn_hup:
                            command: systemctl start cfn-hup.service
        Properties:
            InstanceType: !Ref InstanceType
            SubnetId: !Ref SubnetId
            SecurityGroupIds:
                - !GetAtt InstanceSecurityGroup.GroupId
            KeyName: !Ref KeyName
            ImageId: !FindInMap
                - AWSRegionArch2AMI
                - !Ref AWS::Region
                - !FindInMap
                  - AWSInstanceType2Arch
                  - !Ref InstanceType
                  - Arch
            UserData: !Base64
                Fn::Sub: |
                    #!/bin/bash -xe
                    sudo apt-get update -y
                    sudo apt-get -y install python3-pip
                    mkdir -p /opt/aws/
                    sudo pip3 install https://s3.amazonaws.com/cloudformation-examples/aws-cfn-bootstrap-py3-latest.tar.gz
                    sudo ln -s /usr/local/init/ubuntu/cfn-hup /etc/init.d/cfn-hup
                    /usr/local/bin/cfn-init -v --stack ${AWS::StackName} --resource EC2Instance --configsets full_install --region ${AWS::Region}
                    /usr/local/bin/cfn-signal -e $?  --stack ${AWS::StackName} --resource EC2Instance --region ${AWS::Region}

    InstanceSecurityGroup:
        Type: AWS::EC2::SecurityGroup
        Properties:
            GroupDescription: Enable SSH access via port 22
            SecurityGroupIngress:
                - IpProtocol: tcp
                  FromPort: "22"
                  ToPort: "22"
                  CidrIp: !Ref SSHLocation

`````

### Expected (prettier)

`````yaml
AWSTemplateFormatVersion: "2010-09-09"

Description: Installing Cloudformation helper scripts in Ubuntu 20.04 LTS

Parameters:
    KeyName:
        Description: Name of an existing EC2 KeyPair to enable SSH access to the instance
        Type: AWS::EC2::KeyPair::KeyName

    InstanceType:
        Description: WebServer EC2 instance type
        Type: String
        AllowedValues:
            - t1.micro
            - t2.nano
            - t2.micro
            - t2.small
            - t2.medium
            - t2.large
            - m1.small
            - m1.medium
            - m1.large
            - m1.xlarge
            - m2.xlarge
            - m2.2xlarge
            - m2.4xlarge
            - m3.medium
            - m3.large
            - m3.xlarge
            - m3.2xlarge
            - m4.large
            - m4.xlarge
            - m4.2xlarge
            - m4.4xlarge
            - m4.10xlarge
            - c1.medium
            - c1.xlarge
            - c3.large
            - c3.xlarge
            - c3.2xlarge
            - c3.4xlarge
            - c3.8xlarge
            - c4.large
            - c4.xlarge
            - c4.2xlarge
            - c4.4xlarge
            - c4.8xlarge
            - r3.large
            - r3.xlarge
            - r3.2xlarge
            - r3.4xlarge
            - r3.8xlarge
            - i2.xlarge
            - i2.2xlarge
            - i2.4xlarge
            - i2.8xlarge
            - d2.xlarge
            - d2.2xlarge
            - d2.4xlarge
            - d2.8xlarge
            - hs1.8xlarge
            - cr1.8xlarge
            - cc2.8xlarge
        Default: t2.small
        ConstraintDescription: must be a valid EC2 instance type.

    SSHLocation:
        Description: The IP address range that can be used to SSH to the EC2 instances
        Type: String
        Default: 0.0.0.0/0
        MinLength: "9"
        MaxLength: "18"
        AllowedPattern: (\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})/(\d{1,2})
        ConstraintDescription: must be a valid IP CIDR range of the form x.x.x.x/x.

    SubnetId:
        Type: AWS::EC2::Subnet::Id

Mappings:
    AWSInstanceType2Arch:
        t1.micro:
            Arch: HVM64
        t2.nano:
            Arch: HVM64
        t2.micro:
            Arch: HVM64
        t2.small:
            Arch: HVM64
        t2.medium:
            Arch: HVM64
        t2.large:
            Arch: HVM64
        m1.small:
            Arch: HVM64
        m1.medium:
            Arch: HVM64
        m1.large:
            Arch: HVM64
        m1.xlarge:
            Arch: HVM64
        m2.xlarge:
            Arch: HVM64
        m2.2xlarge:
            Arch: HVM64
        m2.4xlarge:
            Arch: HVM64
        m3.medium:
            Arch: HVM64
        m3.large:
            Arch: HVM64
        m3.xlarge:
            Arch: HVM64
        m3.2xlarge:
            Arch: HVM64
        m4.large:
            Arch: HVM64
        m4.xlarge:
            Arch: HVM64
        m4.2xlarge:
            Arch: HVM64
        m4.4xlarge:
            Arch: HVM64
        m4.10xlarge:
            Arch: HVM64
        c1.medium:
            Arch: HVM64
        c1.xlarge:
            Arch: HVM64
        c3.large:
            Arch: HVM64
        c3.xlarge:
            Arch: HVM64
        c3.2xlarge:
            Arch: HVM64
        c3.4xlarge:
            Arch: HVM64
        c3.8xlarge:
            Arch: HVM64
        c4.large:
            Arch: HVM64
        c4.xlarge:
            Arch: HVM64
        c4.2xlarge:
            Arch: HVM64
        c4.4xlarge:
            Arch: HVM64
        c4.8xlarge:
            Arch: HVM64
        r3.large:
            Arch: HVM64
        r3.xlarge:
            Arch: HVM64
        r3.2xlarge:
            Arch: HVM64
        r3.4xlarge:
            Arch: HVM64
        r3.8xlarge:
            Arch: HVM64
        i2.xlarge:
            Arch: HVM64
        i2.2xlarge:
            Arch: HVM64
        i2.4xlarge:
            Arch: HVM64
        i2.8xlarge:
            Arch: HVM64
        d2.xlarge:
            Arch: HVM64
        d2.2xlarge:
            Arch: HVM64
        d2.4xlarge:
            Arch: HVM64
        d2.8xlarge:
            Arch: HVM64
        hi1.4xlarge:
            Arch: HVM64
        hs1.8xlarge:
            Arch: HVM64
        cr1.8xlarge:
            Arch: HVM64
        cc2.8xlarge:
            Arch: HVM64

    AWSRegionArch2AMI:
        us-east-1:
            HVM64: ami-0149b2da6ceec4bb0
        us-west-2:
            HVM64: ami-0c09c7eb16d3e8e70
        us-west-1:
            HVM64: ami-03f6d497fceb40069
        eu-west-1:
            HVM64: ami-0fd8802f94ed1c969
        eu-west-2:
            HVM64: ami-04842bc62789b682e
        eu-west-3:
            HVM64: ami-064736ff8301af3ee
        eu-central-1:
            HVM64: ami-06148e0e81e5187c8
        ap-northeast-1:
            HVM64: ami-09b18720cb71042df
        ap-northeast-2:
            HVM64: ami-07d16c043aa8e5153
        ap-northeast-3:
            HVM64: ami-09d2f3a31110c6ad4
        ap-southeast-1:
            HVM64: ami-00e912d13fbb4f225
        ap-southeast-2:
            HVM64: ami-055166f8a8041fbf1
        ap-south-1:
            HVM64: ami-024c319d5d14b463e
        us-east-2:
            HVM64: ami-0d5bf08bc8017c83b
        ca-central-1:
            HVM64: ami-043a72cf696697251
        sa-east-1:
            HVM64: ami-00742e66d44c13cd9

Resources:
    EC2Instance:
        CreationPolicy:
            ResourceSignal:
                Timeout: PT10M
                Count: "1"
        Type: AWS::EC2::Instance
        Metadata:
            AWS::CloudFormation::Init:
                configSets:
                    full_install:
                        - install_and_enable_cfn_hup
                install_and_enable_cfn_hup:
                    files:
                        /etc/cfn/cfn-hup.conf:
                            content: !Sub |
                                [main]
                                stack=${AWS::StackId}
                                region=${AWS::Region}
                            mode: "000400"
                            owner: root
                            group: root
                        /etc/cfn/hooks.d/cfn-auto-reloader.conf:
                            content: !Sub |
                                [cfn-auto-reloader-hook]
                                triggers=post.update
                                path=Resources.WebServerInstance.Metadata.AWS::CloudFormation::Init
                                action=/opt/aws/bin/cfn-init -v --stack ${AWS::StackName} --resource WebServerInstance --configsets InstallAndRun --region ${AWS::Region}
                                runas=root
                            mode: "000400"
                            owner: root
                            group: root
                        /lib/systemd/system/cfn-hup.service:
                            content: |
                                [Unit]
                                Description=cfn-hup daemon
                                [Service]
                                Type=simple
                                ExecStart=/usr/local/bin/cfn-hup
                                Restart=always
                                [Install]
                                WantedBy=multi-user.target
                    commands:
                        01enable_cfn_hup:
                            command: systemctl enable cfn-hup.service
                        02start_cfn_hup:
                            command: systemctl start cfn-hup.service
        Properties:
            InstanceType: !Ref InstanceType
            SubnetId: !Ref SubnetId
            SecurityGroupIds:
                - !GetAtt InstanceSecurityGroup.GroupId
            KeyName: !Ref KeyName
            ImageId: !FindInMap
                - AWSRegionArch2AMI
                - !Ref AWS::Region
                - !FindInMap
                  - AWSInstanceType2Arch
                  - !Ref InstanceType
                  - Arch
            UserData: !Base64
                Fn::Sub: |
                    #!/bin/bash -xe
                    sudo apt-get update -y
                    sudo apt-get -y install python3-pip
                    mkdir -p /opt/aws/
                    sudo pip3 install https://s3.amazonaws.com/cloudformation-examples/aws-cfn-bootstrap-py3-latest.tar.gz
                    sudo ln -s /usr/local/init/ubuntu/cfn-hup /etc/init.d/cfn-hup
                    /usr/local/bin/cfn-init -v --stack ${AWS::StackName} --resource EC2Instance --configsets full_install --region ${AWS::Region}
                    /usr/local/bin/cfn-signal -e $?  --stack ${AWS::StackName} --resource EC2Instance --region ${AWS::Region}

    InstanceSecurityGroup:
        Type: AWS::EC2::SecurityGroup
        Properties:
            GroupDescription: Enable SSH access via port 22
            SecurityGroupIngress:
                - IpProtocol: tcp
                  FromPort: "22"
                  ToPort: "22"
                  CidrIp: !Ref SSHLocation

`````

## Option 3

`````json
{"printWidth":120,"singleQuote":true,"bracketSpacing":false,"trailingComma":"none"}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -242,9 +242,9 @@
                 [cfn-auto-reloader-hook]
                 triggers=post.update
                 path=Resources.WebServerInstance.Metadata.AWS::CloudFormation::Init
                 action=/opt/aws/bin/cfn-init -v --stack ${AWS::StackName} --resource WebServerInstance --configsets InstallAndRun --region ${AWS::Region}
-                runas=root
+                runas=root              
               mode: '000400'
               owner: root
               group: root
             /lib/systemd/system/cfn-hup.service:

`````

### Actual (oxfmt)

`````yaml
AWSTemplateFormatVersion: '2010-09-09'

Description: Installing Cloudformation helper scripts in Ubuntu 20.04 LTS

Parameters:
  KeyName:
    Description: Name of an existing EC2 KeyPair to enable SSH access to the instance
    Type: AWS::EC2::KeyPair::KeyName

  InstanceType:
    Description: WebServer EC2 instance type
    Type: String
    AllowedValues:
      - t1.micro
      - t2.nano
      - t2.micro
      - t2.small
      - t2.medium
      - t2.large
      - m1.small
      - m1.medium
      - m1.large
      - m1.xlarge
      - m2.xlarge
      - m2.2xlarge
      - m2.4xlarge
      - m3.medium
      - m3.large
      - m3.xlarge
      - m3.2xlarge
      - m4.large
      - m4.xlarge
      - m4.2xlarge
      - m4.4xlarge
      - m4.10xlarge
      - c1.medium
      - c1.xlarge
      - c3.large
      - c3.xlarge
      - c3.2xlarge
      - c3.4xlarge
      - c3.8xlarge
      - c4.large
      - c4.xlarge
      - c4.2xlarge
      - c4.4xlarge
      - c4.8xlarge
      - r3.large
      - r3.xlarge
      - r3.2xlarge
      - r3.4xlarge
      - r3.8xlarge
      - i2.xlarge
      - i2.2xlarge
      - i2.4xlarge
      - i2.8xlarge
      - d2.xlarge
      - d2.2xlarge
      - d2.4xlarge
      - d2.8xlarge
      - hs1.8xlarge
      - cr1.8xlarge
      - cc2.8xlarge
    Default: t2.small
    ConstraintDescription: must be a valid EC2 instance type.

  SSHLocation:
    Description: The IP address range that can be used to SSH to the EC2 instances
    Type: String
    Default: 0.0.0.0/0
    MinLength: '9'
    MaxLength: '18'
    AllowedPattern: (\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})/(\d{1,2})
    ConstraintDescription: must be a valid IP CIDR range of the form x.x.x.x/x.

  SubnetId:
    Type: AWS::EC2::Subnet::Id

Mappings:
  AWSInstanceType2Arch:
    t1.micro:
      Arch: HVM64
    t2.nano:
      Arch: HVM64
    t2.micro:
      Arch: HVM64
    t2.small:
      Arch: HVM64
    t2.medium:
      Arch: HVM64
    t2.large:
      Arch: HVM64
    m1.small:
      Arch: HVM64
    m1.medium:
      Arch: HVM64
    m1.large:
      Arch: HVM64
    m1.xlarge:
      Arch: HVM64
    m2.xlarge:
      Arch: HVM64
    m2.2xlarge:
      Arch: HVM64
    m2.4xlarge:
      Arch: HVM64
    m3.medium:
      Arch: HVM64
    m3.large:
      Arch: HVM64
    m3.xlarge:
      Arch: HVM64
    m3.2xlarge:
      Arch: HVM64
    m4.large:
      Arch: HVM64
    m4.xlarge:
      Arch: HVM64
    m4.2xlarge:
      Arch: HVM64
    m4.4xlarge:
      Arch: HVM64
    m4.10xlarge:
      Arch: HVM64
    c1.medium:
      Arch: HVM64
    c1.xlarge:
      Arch: HVM64
    c3.large:
      Arch: HVM64
    c3.xlarge:
      Arch: HVM64
    c3.2xlarge:
      Arch: HVM64
    c3.4xlarge:
      Arch: HVM64
    c3.8xlarge:
      Arch: HVM64
    c4.large:
      Arch: HVM64
    c4.xlarge:
      Arch: HVM64
    c4.2xlarge:
      Arch: HVM64
    c4.4xlarge:
      Arch: HVM64
    c4.8xlarge:
      Arch: HVM64
    r3.large:
      Arch: HVM64
    r3.xlarge:
      Arch: HVM64
    r3.2xlarge:
      Arch: HVM64
    r3.4xlarge:
      Arch: HVM64
    r3.8xlarge:
      Arch: HVM64
    i2.xlarge:
      Arch: HVM64
    i2.2xlarge:
      Arch: HVM64
    i2.4xlarge:
      Arch: HVM64
    i2.8xlarge:
      Arch: HVM64
    d2.xlarge:
      Arch: HVM64
    d2.2xlarge:
      Arch: HVM64
    d2.4xlarge:
      Arch: HVM64
    d2.8xlarge:
      Arch: HVM64
    hi1.4xlarge:
      Arch: HVM64
    hs1.8xlarge:
      Arch: HVM64
    cr1.8xlarge:
      Arch: HVM64
    cc2.8xlarge:
      Arch: HVM64

  AWSRegionArch2AMI:
    us-east-1:
      HVM64: ami-0149b2da6ceec4bb0
    us-west-2:
      HVM64: ami-0c09c7eb16d3e8e70
    us-west-1:
      HVM64: ami-03f6d497fceb40069
    eu-west-1:
      HVM64: ami-0fd8802f94ed1c969
    eu-west-2:
      HVM64: ami-04842bc62789b682e
    eu-west-3:
      HVM64: ami-064736ff8301af3ee
    eu-central-1:
      HVM64: ami-06148e0e81e5187c8
    ap-northeast-1:
      HVM64: ami-09b18720cb71042df
    ap-northeast-2:
      HVM64: ami-07d16c043aa8e5153
    ap-northeast-3:
      HVM64: ami-09d2f3a31110c6ad4
    ap-southeast-1:
      HVM64: ami-00e912d13fbb4f225
    ap-southeast-2:
      HVM64: ami-055166f8a8041fbf1
    ap-south-1:
      HVM64: ami-024c319d5d14b463e
    us-east-2:
      HVM64: ami-0d5bf08bc8017c83b
    ca-central-1:
      HVM64: ami-043a72cf696697251
    sa-east-1:
      HVM64: ami-00742e66d44c13cd9

Resources:
  EC2Instance:
    CreationPolicy:
      ResourceSignal:
        Timeout: PT10M
        Count: '1'
    Type: AWS::EC2::Instance
    Metadata:
      AWS::CloudFormation::Init:
        configSets:
          full_install:
            - install_and_enable_cfn_hup
        install_and_enable_cfn_hup:
          files:
            /etc/cfn/cfn-hup.conf:
              content: !Sub |
                [main]
                stack=${AWS::StackId}
                region=${AWS::Region}
              mode: '000400'
              owner: root
              group: root
            /etc/cfn/hooks.d/cfn-auto-reloader.conf:
              content: !Sub |
                [cfn-auto-reloader-hook]
                triggers=post.update
                path=Resources.WebServerInstance.Metadata.AWS::CloudFormation::Init
                action=/opt/aws/bin/cfn-init -v --stack ${AWS::StackName} --resource WebServerInstance --configsets InstallAndRun --region ${AWS::Region}
                runas=root              
              mode: '000400'
              owner: root
              group: root
            /lib/systemd/system/cfn-hup.service:
              content: |
                [Unit]
                Description=cfn-hup daemon
                [Service]
                Type=simple
                ExecStart=/usr/local/bin/cfn-hup
                Restart=always
                [Install]
                WantedBy=multi-user.target
          commands:
            01enable_cfn_hup:
              command: systemctl enable cfn-hup.service
            02start_cfn_hup:
              command: systemctl start cfn-hup.service
    Properties:
      InstanceType: !Ref InstanceType
      SubnetId: !Ref SubnetId
      SecurityGroupIds:
        - !GetAtt InstanceSecurityGroup.GroupId
      KeyName: !Ref KeyName
      ImageId: !FindInMap
        - AWSRegionArch2AMI
        - !Ref AWS::Region
        - !FindInMap
          - AWSInstanceType2Arch
          - !Ref InstanceType
          - Arch
      UserData: !Base64
        Fn::Sub: |
          #!/bin/bash -xe
          sudo apt-get update -y
          sudo apt-get -y install python3-pip
          mkdir -p /opt/aws/
          sudo pip3 install https://s3.amazonaws.com/cloudformation-examples/aws-cfn-bootstrap-py3-latest.tar.gz
          sudo ln -s /usr/local/init/ubuntu/cfn-hup /etc/init.d/cfn-hup
          /usr/local/bin/cfn-init -v --stack ${AWS::StackName} --resource EC2Instance --configsets full_install --region ${AWS::Region}
          /usr/local/bin/cfn-signal -e $?  --stack ${AWS::StackName} --resource EC2Instance --region ${AWS::Region}

  InstanceSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Enable SSH access via port 22
      SecurityGroupIngress:
        - IpProtocol: tcp
          FromPort: '22'
          ToPort: '22'
          CidrIp: !Ref SSHLocation

`````

### Expected (prettier)

`````yaml
AWSTemplateFormatVersion: '2010-09-09'

Description: Installing Cloudformation helper scripts in Ubuntu 20.04 LTS

Parameters:
  KeyName:
    Description: Name of an existing EC2 KeyPair to enable SSH access to the instance
    Type: AWS::EC2::KeyPair::KeyName

  InstanceType:
    Description: WebServer EC2 instance type
    Type: String
    AllowedValues:
      - t1.micro
      - t2.nano
      - t2.micro
      - t2.small
      - t2.medium
      - t2.large
      - m1.small
      - m1.medium
      - m1.large
      - m1.xlarge
      - m2.xlarge
      - m2.2xlarge
      - m2.4xlarge
      - m3.medium
      - m3.large
      - m3.xlarge
      - m3.2xlarge
      - m4.large
      - m4.xlarge
      - m4.2xlarge
      - m4.4xlarge
      - m4.10xlarge
      - c1.medium
      - c1.xlarge
      - c3.large
      - c3.xlarge
      - c3.2xlarge
      - c3.4xlarge
      - c3.8xlarge
      - c4.large
      - c4.xlarge
      - c4.2xlarge
      - c4.4xlarge
      - c4.8xlarge
      - r3.large
      - r3.xlarge
      - r3.2xlarge
      - r3.4xlarge
      - r3.8xlarge
      - i2.xlarge
      - i2.2xlarge
      - i2.4xlarge
      - i2.8xlarge
      - d2.xlarge
      - d2.2xlarge
      - d2.4xlarge
      - d2.8xlarge
      - hs1.8xlarge
      - cr1.8xlarge
      - cc2.8xlarge
    Default: t2.small
    ConstraintDescription: must be a valid EC2 instance type.

  SSHLocation:
    Description: The IP address range that can be used to SSH to the EC2 instances
    Type: String
    Default: 0.0.0.0/0
    MinLength: '9'
    MaxLength: '18'
    AllowedPattern: (\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})/(\d{1,2})
    ConstraintDescription: must be a valid IP CIDR range of the form x.x.x.x/x.

  SubnetId:
    Type: AWS::EC2::Subnet::Id

Mappings:
  AWSInstanceType2Arch:
    t1.micro:
      Arch: HVM64
    t2.nano:
      Arch: HVM64
    t2.micro:
      Arch: HVM64
    t2.small:
      Arch: HVM64
    t2.medium:
      Arch: HVM64
    t2.large:
      Arch: HVM64
    m1.small:
      Arch: HVM64
    m1.medium:
      Arch: HVM64
    m1.large:
      Arch: HVM64
    m1.xlarge:
      Arch: HVM64
    m2.xlarge:
      Arch: HVM64
    m2.2xlarge:
      Arch: HVM64
    m2.4xlarge:
      Arch: HVM64
    m3.medium:
      Arch: HVM64
    m3.large:
      Arch: HVM64
    m3.xlarge:
      Arch: HVM64
    m3.2xlarge:
      Arch: HVM64
    m4.large:
      Arch: HVM64
    m4.xlarge:
      Arch: HVM64
    m4.2xlarge:
      Arch: HVM64
    m4.4xlarge:
      Arch: HVM64
    m4.10xlarge:
      Arch: HVM64
    c1.medium:
      Arch: HVM64
    c1.xlarge:
      Arch: HVM64
    c3.large:
      Arch: HVM64
    c3.xlarge:
      Arch: HVM64
    c3.2xlarge:
      Arch: HVM64
    c3.4xlarge:
      Arch: HVM64
    c3.8xlarge:
      Arch: HVM64
    c4.large:
      Arch: HVM64
    c4.xlarge:
      Arch: HVM64
    c4.2xlarge:
      Arch: HVM64
    c4.4xlarge:
      Arch: HVM64
    c4.8xlarge:
      Arch: HVM64
    r3.large:
      Arch: HVM64
    r3.xlarge:
      Arch: HVM64
    r3.2xlarge:
      Arch: HVM64
    r3.4xlarge:
      Arch: HVM64
    r3.8xlarge:
      Arch: HVM64
    i2.xlarge:
      Arch: HVM64
    i2.2xlarge:
      Arch: HVM64
    i2.4xlarge:
      Arch: HVM64
    i2.8xlarge:
      Arch: HVM64
    d2.xlarge:
      Arch: HVM64
    d2.2xlarge:
      Arch: HVM64
    d2.4xlarge:
      Arch: HVM64
    d2.8xlarge:
      Arch: HVM64
    hi1.4xlarge:
      Arch: HVM64
    hs1.8xlarge:
      Arch: HVM64
    cr1.8xlarge:
      Arch: HVM64
    cc2.8xlarge:
      Arch: HVM64

  AWSRegionArch2AMI:
    us-east-1:
      HVM64: ami-0149b2da6ceec4bb0
    us-west-2:
      HVM64: ami-0c09c7eb16d3e8e70
    us-west-1:
      HVM64: ami-03f6d497fceb40069
    eu-west-1:
      HVM64: ami-0fd8802f94ed1c969
    eu-west-2:
      HVM64: ami-04842bc62789b682e
    eu-west-3:
      HVM64: ami-064736ff8301af3ee
    eu-central-1:
      HVM64: ami-06148e0e81e5187c8
    ap-northeast-1:
      HVM64: ami-09b18720cb71042df
    ap-northeast-2:
      HVM64: ami-07d16c043aa8e5153
    ap-northeast-3:
      HVM64: ami-09d2f3a31110c6ad4
    ap-southeast-1:
      HVM64: ami-00e912d13fbb4f225
    ap-southeast-2:
      HVM64: ami-055166f8a8041fbf1
    ap-south-1:
      HVM64: ami-024c319d5d14b463e
    us-east-2:
      HVM64: ami-0d5bf08bc8017c83b
    ca-central-1:
      HVM64: ami-043a72cf696697251
    sa-east-1:
      HVM64: ami-00742e66d44c13cd9

Resources:
  EC2Instance:
    CreationPolicy:
      ResourceSignal:
        Timeout: PT10M
        Count: '1'
    Type: AWS::EC2::Instance
    Metadata:
      AWS::CloudFormation::Init:
        configSets:
          full_install:
            - install_and_enable_cfn_hup
        install_and_enable_cfn_hup:
          files:
            /etc/cfn/cfn-hup.conf:
              content: !Sub |
                [main]
                stack=${AWS::StackId}
                region=${AWS::Region}
              mode: '000400'
              owner: root
              group: root
            /etc/cfn/hooks.d/cfn-auto-reloader.conf:
              content: !Sub |
                [cfn-auto-reloader-hook]
                triggers=post.update
                path=Resources.WebServerInstance.Metadata.AWS::CloudFormation::Init
                action=/opt/aws/bin/cfn-init -v --stack ${AWS::StackName} --resource WebServerInstance --configsets InstallAndRun --region ${AWS::Region}
                runas=root
              mode: '000400'
              owner: root
              group: root
            /lib/systemd/system/cfn-hup.service:
              content: |
                [Unit]
                Description=cfn-hup daemon
                [Service]
                Type=simple
                ExecStart=/usr/local/bin/cfn-hup
                Restart=always
                [Install]
                WantedBy=multi-user.target
          commands:
            01enable_cfn_hup:
              command: systemctl enable cfn-hup.service
            02start_cfn_hup:
              command: systemctl start cfn-hup.service
    Properties:
      InstanceType: !Ref InstanceType
      SubnetId: !Ref SubnetId
      SecurityGroupIds:
        - !GetAtt InstanceSecurityGroup.GroupId
      KeyName: !Ref KeyName
      ImageId: !FindInMap
        - AWSRegionArch2AMI
        - !Ref AWS::Region
        - !FindInMap
          - AWSInstanceType2Arch
          - !Ref InstanceType
          - Arch
      UserData: !Base64
        Fn::Sub: |
          #!/bin/bash -xe
          sudo apt-get update -y
          sudo apt-get -y install python3-pip
          mkdir -p /opt/aws/
          sudo pip3 install https://s3.amazonaws.com/cloudformation-examples/aws-cfn-bootstrap-py3-latest.tar.gz
          sudo ln -s /usr/local/init/ubuntu/cfn-hup /etc/init.d/cfn-hup
          /usr/local/bin/cfn-init -v --stack ${AWS::StackName} --resource EC2Instance --configsets full_install --region ${AWS::Region}
          /usr/local/bin/cfn-signal -e $?  --stack ${AWS::StackName} --resource EC2Instance --region ${AWS::Region}

  InstanceSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Enable SSH access via port 22
      SecurityGroupIngress:
        - IpProtocol: tcp
          FromPort: '22'
          ToPort: '22'
          CidrIp: !Ref SSHLocation

`````
