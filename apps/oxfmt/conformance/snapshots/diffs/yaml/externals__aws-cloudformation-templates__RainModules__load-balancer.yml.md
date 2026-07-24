# externals/aws-cloudformation-templates/RainModules/load-balancer.yml

> Allowed: over-indented comment after `key: value` (Prettier breaks the pair onto two lines because of comment indentation). See crates/oxc_formatter_yaml/AGENTS.md

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -40,10 +40,9 @@
             - GroupId
       Subnets:
         - !Ref PublicSubnet1
         - !Ref PublicSubnet2
-      Type:
-        application
+      Type: application
         # Need these... but can't put them in the module
         # They will need to be overrides in the parent which is not ideal
         #DependsOn:
         #  - PublicSubnet1DefaultRoute

`````

### Actual (oxfmt)

`````yml
Description: |
  This module creates an ELBv2 load balancer

Parameters:
  CertificateArn:
    Type: String

  VPCId:
    Type: String

  PublicSubnet1:
    Type: String

  PublicSubnet2:
    Type: String

  DestinationSecurityGroupId:
    Type: String

Resources:
  LoadBalancer:
    Type: AWS::ElasticLoadBalancingV2::LoadBalancer
    Metadata:
      checkov:
        skip:
          - id: CKV_AWS_91
      guard:
        SuppressedRules:
          - ELB_DELETION_PROTECTION_ENABLED
    Properties:
      LoadBalancerAttributes:
        - Key: deletion_protection.enabled
          Value: false
        - Key: routing.http.drop_invalid_header_fields.enabled
          Value: true
      Scheme: internet-facing
      SecurityGroups:
        - Fn::GetAtt:
            - LoadBalancerSecurityGroup
            - GroupId
      Subnets:
        - !Ref PublicSubnet1
        - !Ref PublicSubnet2
      Type: application
        # Need these... but can't put them in the module
        # They will need to be overrides in the parent which is not ideal
        #DependsOn:
        #  - PublicSubnet1DefaultRoute
        #  - PublicSubnet1RouteTableAssociation
        #  - PublicSubnet2DefaultRoute
        #  - PublicSubnet2RouteTableAssociation

  LoadBalancerSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Automatically created Security Group for ELB
      SecurityGroupIngress:
        - CidrIp: 0.0.0.0/0
          Description: Allow from anyone on port 443
          FromPort: 443
          IpProtocol: tcp
          ToPort: 443
      VpcId: !Ref VPCId

  LoadBalancerEgress:
    Type: AWS::EC2::SecurityGroupEgress
    Properties:
      Description: Load balancer to target
      DestinationSecurityGroupId: !Ref DestinationSecurityGroupId
      FromPort: 80
      GroupId: !GetAtt LoadBalancerSecurityGroup.GroupId
      IpProtocol: tcp
      ToPort: 80

  LoadBalancerListener:
    Type: AWS::ElasticLoadBalancingV2::Listener
    Metadata:
      guard:
        SuppressedRules:
          - ELBV2_ACM_CERTIFICATE_REQUIRED
    Properties:
      DefaultActions:
        - TargetGroupArn: !Ref TargetGroup
          Type: forward
      LoadBalancerArn: !Ref LoadBalancer
      Port: 443
      Protocol: HTTPS
      Certificates:
        - CertificateArn: !Ref CertificateArn
      SslPolicy: ELBSecurityPolicy-TLS13-1-2-2021-06

  TargetGroup:
    Type: AWS::ElasticLoadBalancingV2::TargetGroup
    Properties:
      Port: 80
      Protocol: HTTP
      TargetGroupAttributes:
        - Key: deregistration_delay.timeout_seconds
          Value: "10"
        - Key: stickiness.enabled
          Value: "false"
      TargetType: ip
      VpcId: !Ref VPCId

Outputs:
  LoadBalancerDNS:
    Value: !GetAtt LoadBalancer.DNSName

`````

### Expected (prettier)

`````yml
Description: |
  This module creates an ELBv2 load balancer

Parameters:
  CertificateArn:
    Type: String

  VPCId:
    Type: String

  PublicSubnet1:
    Type: String

  PublicSubnet2:
    Type: String

  DestinationSecurityGroupId:
    Type: String

Resources:
  LoadBalancer:
    Type: AWS::ElasticLoadBalancingV2::LoadBalancer
    Metadata:
      checkov:
        skip:
          - id: CKV_AWS_91
      guard:
        SuppressedRules:
          - ELB_DELETION_PROTECTION_ENABLED
    Properties:
      LoadBalancerAttributes:
        - Key: deletion_protection.enabled
          Value: false
        - Key: routing.http.drop_invalid_header_fields.enabled
          Value: true
      Scheme: internet-facing
      SecurityGroups:
        - Fn::GetAtt:
            - LoadBalancerSecurityGroup
            - GroupId
      Subnets:
        - !Ref PublicSubnet1
        - !Ref PublicSubnet2
      Type:
        application
        # Need these... but can't put them in the module
        # They will need to be overrides in the parent which is not ideal
        #DependsOn:
        #  - PublicSubnet1DefaultRoute
        #  - PublicSubnet1RouteTableAssociation
        #  - PublicSubnet2DefaultRoute
        #  - PublicSubnet2RouteTableAssociation

  LoadBalancerSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Automatically created Security Group for ELB
      SecurityGroupIngress:
        - CidrIp: 0.0.0.0/0
          Description: Allow from anyone on port 443
          FromPort: 443
          IpProtocol: tcp
          ToPort: 443
      VpcId: !Ref VPCId

  LoadBalancerEgress:
    Type: AWS::EC2::SecurityGroupEgress
    Properties:
      Description: Load balancer to target
      DestinationSecurityGroupId: !Ref DestinationSecurityGroupId
      FromPort: 80
      GroupId: !GetAtt LoadBalancerSecurityGroup.GroupId
      IpProtocol: tcp
      ToPort: 80

  LoadBalancerListener:
    Type: AWS::ElasticLoadBalancingV2::Listener
    Metadata:
      guard:
        SuppressedRules:
          - ELBV2_ACM_CERTIFICATE_REQUIRED
    Properties:
      DefaultActions:
        - TargetGroupArn: !Ref TargetGroup
          Type: forward
      LoadBalancerArn: !Ref LoadBalancer
      Port: 443
      Protocol: HTTPS
      Certificates:
        - CertificateArn: !Ref CertificateArn
      SslPolicy: ELBSecurityPolicy-TLS13-1-2-2021-06

  TargetGroup:
    Type: AWS::ElasticLoadBalancingV2::TargetGroup
    Properties:
      Port: 80
      Protocol: HTTP
      TargetGroupAttributes:
        - Key: deregistration_delay.timeout_seconds
          Value: "10"
        - Key: stickiness.enabled
          Value: "false"
      TargetType: ip
      VpcId: !Ref VPCId

Outputs:
  LoadBalancerDNS:
    Value: !GetAtt LoadBalancer.DNSName

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
@@ -40,10 +40,9 @@
                       - GroupId
             Subnets:
                 - !Ref PublicSubnet1
                 - !Ref PublicSubnet2
-            Type:
-                application
+            Type: application
                 # Need these... but can't put them in the module
                 # They will need to be overrides in the parent which is not ideal
                 #DependsOn:
                 #  - PublicSubnet1DefaultRoute

`````

### Actual (oxfmt)

`````yml
Description: |
    This module creates an ELBv2 load balancer

Parameters:
    CertificateArn:
        Type: String

    VPCId:
        Type: String

    PublicSubnet1:
        Type: String

    PublicSubnet2:
        Type: String

    DestinationSecurityGroupId:
        Type: String

Resources:
    LoadBalancer:
        Type: AWS::ElasticLoadBalancingV2::LoadBalancer
        Metadata:
            checkov:
                skip:
                    - id: CKV_AWS_91
            guard:
                SuppressedRules:
                    - ELB_DELETION_PROTECTION_ENABLED
        Properties:
            LoadBalancerAttributes:
                - Key: deletion_protection.enabled
                  Value: false
                - Key: routing.http.drop_invalid_header_fields.enabled
                  Value: true
            Scheme: internet-facing
            SecurityGroups:
                - Fn::GetAtt:
                      - LoadBalancerSecurityGroup
                      - GroupId
            Subnets:
                - !Ref PublicSubnet1
                - !Ref PublicSubnet2
            Type: application
                # Need these... but can't put them in the module
                # They will need to be overrides in the parent which is not ideal
                #DependsOn:
                #  - PublicSubnet1DefaultRoute
                #  - PublicSubnet1RouteTableAssociation
                #  - PublicSubnet2DefaultRoute
                #  - PublicSubnet2RouteTableAssociation

    LoadBalancerSecurityGroup:
        Type: AWS::EC2::SecurityGroup
        Properties:
            GroupDescription: Automatically created Security Group for ELB
            SecurityGroupIngress:
                - CidrIp: 0.0.0.0/0
                  Description: Allow from anyone on port 443
                  FromPort: 443
                  IpProtocol: tcp
                  ToPort: 443
            VpcId: !Ref VPCId

    LoadBalancerEgress:
        Type: AWS::EC2::SecurityGroupEgress
        Properties:
            Description: Load balancer to target
            DestinationSecurityGroupId: !Ref DestinationSecurityGroupId
            FromPort: 80
            GroupId: !GetAtt LoadBalancerSecurityGroup.GroupId
            IpProtocol: tcp
            ToPort: 80

    LoadBalancerListener:
        Type: AWS::ElasticLoadBalancingV2::Listener
        Metadata:
            guard:
                SuppressedRules:
                    - ELBV2_ACM_CERTIFICATE_REQUIRED
        Properties:
            DefaultActions:
                - TargetGroupArn: !Ref TargetGroup
                  Type: forward
            LoadBalancerArn: !Ref LoadBalancer
            Port: 443
            Protocol: HTTPS
            Certificates:
                - CertificateArn: !Ref CertificateArn
            SslPolicy: ELBSecurityPolicy-TLS13-1-2-2021-06

    TargetGroup:
        Type: AWS::ElasticLoadBalancingV2::TargetGroup
        Properties:
            Port: 80
            Protocol: HTTP
            TargetGroupAttributes:
                - Key: deregistration_delay.timeout_seconds
                  Value: "10"
                - Key: stickiness.enabled
                  Value: "false"
            TargetType: ip
            VpcId: !Ref VPCId

Outputs:
    LoadBalancerDNS:
        Value: !GetAtt LoadBalancer.DNSName

`````

### Expected (prettier)

`````yml
Description: |
    This module creates an ELBv2 load balancer

Parameters:
    CertificateArn:
        Type: String

    VPCId:
        Type: String

    PublicSubnet1:
        Type: String

    PublicSubnet2:
        Type: String

    DestinationSecurityGroupId:
        Type: String

Resources:
    LoadBalancer:
        Type: AWS::ElasticLoadBalancingV2::LoadBalancer
        Metadata:
            checkov:
                skip:
                    - id: CKV_AWS_91
            guard:
                SuppressedRules:
                    - ELB_DELETION_PROTECTION_ENABLED
        Properties:
            LoadBalancerAttributes:
                - Key: deletion_protection.enabled
                  Value: false
                - Key: routing.http.drop_invalid_header_fields.enabled
                  Value: true
            Scheme: internet-facing
            SecurityGroups:
                - Fn::GetAtt:
                      - LoadBalancerSecurityGroup
                      - GroupId
            Subnets:
                - !Ref PublicSubnet1
                - !Ref PublicSubnet2
            Type:
                application
                # Need these... but can't put them in the module
                # They will need to be overrides in the parent which is not ideal
                #DependsOn:
                #  - PublicSubnet1DefaultRoute
                #  - PublicSubnet1RouteTableAssociation
                #  - PublicSubnet2DefaultRoute
                #  - PublicSubnet2RouteTableAssociation

    LoadBalancerSecurityGroup:
        Type: AWS::EC2::SecurityGroup
        Properties:
            GroupDescription: Automatically created Security Group for ELB
            SecurityGroupIngress:
                - CidrIp: 0.0.0.0/0
                  Description: Allow from anyone on port 443
                  FromPort: 443
                  IpProtocol: tcp
                  ToPort: 443
            VpcId: !Ref VPCId

    LoadBalancerEgress:
        Type: AWS::EC2::SecurityGroupEgress
        Properties:
            Description: Load balancer to target
            DestinationSecurityGroupId: !Ref DestinationSecurityGroupId
            FromPort: 80
            GroupId: !GetAtt LoadBalancerSecurityGroup.GroupId
            IpProtocol: tcp
            ToPort: 80

    LoadBalancerListener:
        Type: AWS::ElasticLoadBalancingV2::Listener
        Metadata:
            guard:
                SuppressedRules:
                    - ELBV2_ACM_CERTIFICATE_REQUIRED
        Properties:
            DefaultActions:
                - TargetGroupArn: !Ref TargetGroup
                  Type: forward
            LoadBalancerArn: !Ref LoadBalancer
            Port: 443
            Protocol: HTTPS
            Certificates:
                - CertificateArn: !Ref CertificateArn
            SslPolicy: ELBSecurityPolicy-TLS13-1-2-2021-06

    TargetGroup:
        Type: AWS::ElasticLoadBalancingV2::TargetGroup
        Properties:
            Port: 80
            Protocol: HTTP
            TargetGroupAttributes:
                - Key: deregistration_delay.timeout_seconds
                  Value: "10"
                - Key: stickiness.enabled
                  Value: "false"
            TargetType: ip
            VpcId: !Ref VPCId

Outputs:
    LoadBalancerDNS:
        Value: !GetAtt LoadBalancer.DNSName

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
@@ -40,10 +40,9 @@
             - GroupId
       Subnets:
         - !Ref PublicSubnet1
         - !Ref PublicSubnet2
-      Type:
-        application
+      Type: application
         # Need these... but can't put them in the module
         # They will need to be overrides in the parent which is not ideal
         #DependsOn:
         #  - PublicSubnet1DefaultRoute

`````

### Actual (oxfmt)

`````yml
Description: |
  This module creates an ELBv2 load balancer

Parameters:
  CertificateArn:
    Type: String

  VPCId:
    Type: String

  PublicSubnet1:
    Type: String

  PublicSubnet2:
    Type: String

  DestinationSecurityGroupId:
    Type: String

Resources:
  LoadBalancer:
    Type: AWS::ElasticLoadBalancingV2::LoadBalancer
    Metadata:
      checkov:
        skip:
          - id: CKV_AWS_91
      guard:
        SuppressedRules:
          - ELB_DELETION_PROTECTION_ENABLED
    Properties:
      LoadBalancerAttributes:
        - Key: deletion_protection.enabled
          Value: false
        - Key: routing.http.drop_invalid_header_fields.enabled
          Value: true
      Scheme: internet-facing
      SecurityGroups:
        - Fn::GetAtt:
            - LoadBalancerSecurityGroup
            - GroupId
      Subnets:
        - !Ref PublicSubnet1
        - !Ref PublicSubnet2
      Type: application
        # Need these... but can't put them in the module
        # They will need to be overrides in the parent which is not ideal
        #DependsOn:
        #  - PublicSubnet1DefaultRoute
        #  - PublicSubnet1RouteTableAssociation
        #  - PublicSubnet2DefaultRoute
        #  - PublicSubnet2RouteTableAssociation

  LoadBalancerSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Automatically created Security Group for ELB
      SecurityGroupIngress:
        - CidrIp: 0.0.0.0/0
          Description: Allow from anyone on port 443
          FromPort: 443
          IpProtocol: tcp
          ToPort: 443
      VpcId: !Ref VPCId

  LoadBalancerEgress:
    Type: AWS::EC2::SecurityGroupEgress
    Properties:
      Description: Load balancer to target
      DestinationSecurityGroupId: !Ref DestinationSecurityGroupId
      FromPort: 80
      GroupId: !GetAtt LoadBalancerSecurityGroup.GroupId
      IpProtocol: tcp
      ToPort: 80

  LoadBalancerListener:
    Type: AWS::ElasticLoadBalancingV2::Listener
    Metadata:
      guard:
        SuppressedRules:
          - ELBV2_ACM_CERTIFICATE_REQUIRED
    Properties:
      DefaultActions:
        - TargetGroupArn: !Ref TargetGroup
          Type: forward
      LoadBalancerArn: !Ref LoadBalancer
      Port: 443
      Protocol: HTTPS
      Certificates:
        - CertificateArn: !Ref CertificateArn
      SslPolicy: ELBSecurityPolicy-TLS13-1-2-2021-06

  TargetGroup:
    Type: AWS::ElasticLoadBalancingV2::TargetGroup
    Properties:
      Port: 80
      Protocol: HTTP
      TargetGroupAttributes:
        - Key: deregistration_delay.timeout_seconds
          Value: '10'
        - Key: stickiness.enabled
          Value: 'false'
      TargetType: ip
      VpcId: !Ref VPCId

Outputs:
  LoadBalancerDNS:
    Value: !GetAtt LoadBalancer.DNSName

`````

### Expected (prettier)

`````yml
Description: |
  This module creates an ELBv2 load balancer

Parameters:
  CertificateArn:
    Type: String

  VPCId:
    Type: String

  PublicSubnet1:
    Type: String

  PublicSubnet2:
    Type: String

  DestinationSecurityGroupId:
    Type: String

Resources:
  LoadBalancer:
    Type: AWS::ElasticLoadBalancingV2::LoadBalancer
    Metadata:
      checkov:
        skip:
          - id: CKV_AWS_91
      guard:
        SuppressedRules:
          - ELB_DELETION_PROTECTION_ENABLED
    Properties:
      LoadBalancerAttributes:
        - Key: deletion_protection.enabled
          Value: false
        - Key: routing.http.drop_invalid_header_fields.enabled
          Value: true
      Scheme: internet-facing
      SecurityGroups:
        - Fn::GetAtt:
            - LoadBalancerSecurityGroup
            - GroupId
      Subnets:
        - !Ref PublicSubnet1
        - !Ref PublicSubnet2
      Type:
        application
        # Need these... but can't put them in the module
        # They will need to be overrides in the parent which is not ideal
        #DependsOn:
        #  - PublicSubnet1DefaultRoute
        #  - PublicSubnet1RouteTableAssociation
        #  - PublicSubnet2DefaultRoute
        #  - PublicSubnet2RouteTableAssociation

  LoadBalancerSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Automatically created Security Group for ELB
      SecurityGroupIngress:
        - CidrIp: 0.0.0.0/0
          Description: Allow from anyone on port 443
          FromPort: 443
          IpProtocol: tcp
          ToPort: 443
      VpcId: !Ref VPCId

  LoadBalancerEgress:
    Type: AWS::EC2::SecurityGroupEgress
    Properties:
      Description: Load balancer to target
      DestinationSecurityGroupId: !Ref DestinationSecurityGroupId
      FromPort: 80
      GroupId: !GetAtt LoadBalancerSecurityGroup.GroupId
      IpProtocol: tcp
      ToPort: 80

  LoadBalancerListener:
    Type: AWS::ElasticLoadBalancingV2::Listener
    Metadata:
      guard:
        SuppressedRules:
          - ELBV2_ACM_CERTIFICATE_REQUIRED
    Properties:
      DefaultActions:
        - TargetGroupArn: !Ref TargetGroup
          Type: forward
      LoadBalancerArn: !Ref LoadBalancer
      Port: 443
      Protocol: HTTPS
      Certificates:
        - CertificateArn: !Ref CertificateArn
      SslPolicy: ELBSecurityPolicy-TLS13-1-2-2021-06

  TargetGroup:
    Type: AWS::ElasticLoadBalancingV2::TargetGroup
    Properties:
      Port: 80
      Protocol: HTTP
      TargetGroupAttributes:
        - Key: deregistration_delay.timeout_seconds
          Value: '10'
        - Key: stickiness.enabled
          Value: 'false'
      TargetType: ip
      VpcId: !Ref VPCId

Outputs:
  LoadBalancerDNS:
    Value: !GetAtt LoadBalancer.DNSName

`````
