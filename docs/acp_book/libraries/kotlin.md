# Kotlin

> Kotlin library for the Agent Client Protocol

The [kotlin-sdk](https://github.com/agentclientprotocol/kotlin-sdk) provides implementations of both sides of the Agent Client Protocol that
you can use to build your own agent server or client.

**It currently supports JVM, other targets are in progress.**

To get started, add the repository to your build file:

```kotlin  theme={null}
repositories {
    mavenCentral()
}
```

Add the dependency:

```kotlin  theme={null}
dependencies {
    implementation("com.agentclientprotocol:acp:0.1.0-SNAPSHOT")
}
```

The [sample](https://github.com/agentclientprotocol/kotlin-sdk/tree/master/samples/kotlin-acp-client-sample) demonstrates how to implement both sides of the protocol.


---

> To find navigation and other pages in this documentation, fetch the llms.txt file at: https://agentclientprotocol.com/llms.txt