{{- define "image-tag" -}}
{{- if .Values.image.tag -}}
  {{- .Values.image.tag -}}
{{- else -}}
  {{- .Chart.AppVersion -}}
{{- end -}}
{{- end -}}

{{/*
Expand the name of the chart.
*/}}
{{- define "lockbox.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "lockbox.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "lockbox.labels" -}}
helm.sh/chart: {{ include "lockbox.chart" . }}
{{ include "lockbox.selectorLabels" . }}
{{- if (include "image-tag" .) }}
app.kubernetes.io/version: {{ include "image-tag" . | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Server component labels
*/}}
{{- define "lockbox-server.labels" -}}
{{ include "lockbox.labels" . }}
app.kubernetes.io/component: server
{{- end }}

{{/*
Selector labels
*/}}
{{- define "lockbox.selectorLabels" -}}
app.kubernetes.io/name: {{ include "lockbox.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Service selector labels
*/}}
{{- define "lockbox.serviceSelectorLabels" -}}
{{ include "lockbox.selectorLabels" . }}
app.kubernetes.io/component: server
{{- end }}

{{/*
Database environment variables
*/}}
{{- define "lockbox.databaseEnvVars" -}}
{{- $cfg := .config -}}
{{- $opts := .options -}}
{{- $replica_dsns := list -}}
{{- if eq $cfg.primary.type "postgresql" }}
- name: PRIMARY_DSN_USERNAME
  {{- if hasKey $cfg.primary.username "secretKeyRef" }}
  valueFrom:
    secretKeyRef:
      name: {{ $cfg.primary.username.secretKeyRef.name }}
      key: {{ $cfg.primary.username.secretKeyRef.key }}
  {{- else if hasKey $cfg.primary.username "value" }}
  value: {{ $cfg.primary.username.value | quote }}
  {{- end }}
- name: PRIMARY_DSN_PASSWORD
  {{- if hasKey $cfg.primary.password "secretKeyRef" }}
  valueFrom:
    secretKeyRef:
      name: {{ $cfg.primary.password.secretKeyRef.name }}
      key: {{ $cfg.primary.password.secretKeyRef.key }}
  {{- else if hasKey $cfg.primary.password "value" }}
  value: {{ $cfg.primary.password.value | quote }}
  {{- end }}
- name: LOCKBOX__DATABASE__PRIMARY
  value: "postgresql://$(PRIMARY_DSN_USERNAME):$(PRIMARY_DSN_PASSWORD)@{{ $cfg.primary.host }}:{{ (int $cfg.primary.port) }}/{{ $cfg.primary.name }}"
{{- else if eq $cfg.primary.type "sqlite" }}
- name: LOCKBOX__DATABASE__PRIMARY
  value: "sqlite:///{{ $cfg.primary.path }}"
{{- end }}
{{- range $index, $replica := $cfg.replicas }}
{{- if eq $replica.type "postgresql" }}
- name: REPLICA_{{ $index }}_DSN_USERNAME
  {{- if hasKey $replica.username "secretKeyRef" }}
  valueFrom:
    secretKeyRef:
      name: {{ $replica.username.secretKeyRef.name }}
      key: {{ $replica.username.secretKeyRef.key }}
  {{- else if hasKey $replica.username "value" }}
  value: {{ $replica.username.value | quote }}
  {{- end }}
- name: REPLICA_{{ $index }}_DSN_PASSWORD
  {{- if hasKey $replica.password "secretKeyRef" }}
  valueFrom:
    secretKeyRef:
      name: {{ $replica.password.secretKeyRef.name }}
      key: {{ $replica.password.secretKeyRef.key }}
  {{- else if hasKey $replica.password "value" }}
  value: {{ $replica.password.value | quote }}
  {{- end }}
{{- $dsn := printf "postgresql://$(REPLICA_%d_DSN_USERNAME):$(REPLICA_%d_DSN_PASSWORD)@%s:%d/%s" $index $index $replica.host (int $replica.port) $replica.name }}
{{- $replica_dsns = append $replica_dsns $dsn }}
{{- else if eq $replica.type "sqlite" }}
{{- $dsn := printf "sqlite:///%s" $replica.path }}
{{- $replica_dsns = append $replica_dsns $dsn }}
{{- end }}
{{- end }}
{{- range $index, $dsn := $replica_dsns }}
- name: LOCKBOX__DATABASE__REPLICAS__{{ $index }}
  value: {{ $dsn | quote }}
{{- end }}
{{- range $key, $value := $opts }}
- name: {{ printf "LOCKBOX__DATABASE__OPTIONS__%s" ($key | lower) }}
  value: {{ $value | quote }}
{{- end }}
{{- end }}


{{/*
Bootstrap script
*/}}
{{- define "lockbox.bootstrapScript" -}}
{{- $bootstrap := .bootstrap -}}
{{- $config := .config -}}
set -e
{{- $defaultTenant := $config.defaultTenantId }}
{{- range $ns := $bootstrap.namespaces }}
{{- $tenantId := default $defaultTenant $ns.tenantId }}
{{- if $ns.name }}
lockbox namespaces{{ if $tenantId }} --tenant-id {{ $tenantId }}{{ end }} create --skip-exists {{ $ns.name }}
{{- end }}
{{- range $tag := $ns.tags }}
{{- if (and $tag $ns.name ) }}
lockbox tags{{ if $tenantId }} --tenant-id {{ $tenantId }}{{ end }} --namespace {{ $ns.name }} create --skip-exists {{ $tag }}
{{- end}}
{{- end }}
{{- end }}
{{- end }}