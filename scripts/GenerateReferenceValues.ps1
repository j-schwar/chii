param(
    [Parameter(Mandatory = $true)]
    [string] $Path,

    [Parameter(Mandatory = $true)]
    [string] $Schema,

    [Parameter(Mandatory = $true)]
    [string] $Object
)

function Convert-Hex {
    param(
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [string] $Text
    )

    $encoder = [System.Text.Encoding]::UTF8
    $bytes = $encoder.GetBytes($Text) | ForEach-Object { [System.String]::Format("{0:X2}", $_) }
    return $bytes -Join ""
}

function Get-EncodedSize {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Path,

        [Parameter(Mandatory = $true)]
        [string] $Schema,

        [Parameter(Mandatory = $true)]
        [string] $Object,

        [Parameter(Mandatory = $true)]
        [string] $EncodingRule
    )

    $encoded = Get-Content $Path |
        ConvertFrom-Json |
            ConvertTo-Json -Compress |
                Convert-Hex |
                    asn1tools.exe convert -i jer -o $EncodingRule $Schema $Object -
    return $encoded.Length / 2
}

function Get-GZipSize {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Path
    )

    $compressed = $Path + ".gz"
    gzip.exe -kn9f $Path
    return (Get-Content -Encoding Byte $compressed).Length
}

# Control
$jsonResult = [PSCustomObject]@{
    Encoding = "json";
    Size = (Get-Content $Path).Length
}

# GZip Compression
$gzipResult = [PSCustomObject]@{
    Encoding = "json + gzip -n9"
    Size = Get-GZipSize($Path)
}

# ASN.1 Encodings
$rules = "der", "per", "uper"
$asn1Results = $rules | ForEach-Object {
    $size = Get-EncodedSize $Path $Schema $Object -EncodingRule $_
    return [PSCustomObject]@{
        Encoding = $_;
        Size = $size;
    }
}

# Print Result
@($jsonResult) + @($gzipResult) + $asn1Results | Format-Table
