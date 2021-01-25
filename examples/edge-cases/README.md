# KHR_xmp_json_ld Edge Case Examples

These examples are provided for tools to test edge cases with the `KHR_xmp_json_ld` extension. Only `.gltf` versions are provided.

## Cases

### Language Alternatives (language-alternatives)

This example includes a simple use of the _Language Alternatives_ feature of XMP.

### Non-alternative Language Text using Language Attribute (language-attribute)

In some cases, XMP data can provide a language indication when the data type is not a _Language Alternative_, such as with the Dublin Core `dc:subject` or `dc:source` fields. This may involves the use of a value object as per the ISO specification: [ISO/DIS 16684-3](https://www.iso.org/standard/79384.html).
