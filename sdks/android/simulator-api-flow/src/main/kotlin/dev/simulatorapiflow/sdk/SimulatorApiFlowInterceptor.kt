package dev.simulatorapiflow.sdk

import okhttp3.Interceptor
import okhttp3.Response

/**
 * Opt-in OkHttp interceptor that adds SimulatorApiFlow correlation metadata.
 * When the SDK is disabled the original request passes through untouched.
 *
 * Source file/function metadata should be supplied by [SimulatorApiFlow.setContext] at the
 * feature boundary. The interceptor intentionally does not infer a source location because
 * its call stack is dominated by OkHttp internals rather than the app call site.
 */
public class SimulatorApiFlowInterceptor(
    private val feature: String? = null,
    private val attributes: () -> Map<String, String> = { emptyMap() },
) : Interceptor {
    override fun intercept(chain: Interceptor.Chain): Response {
        val original = chain.request()
        val instrumented = SimulatorApiFlow.instrument(
            request = original,
            feature = feature,
            attributes = attributes(),
            source = null,
        )

        if (instrumented == null) {
            return chain.proceed(original)
        }

        return try {
            val response = chain.proceed(instrumented.request)
            SimulatorApiFlow.complete(
                requestId = instrumented.requestId,
                response = response,
            )
            response
        } catch (throwable: Throwable) {
            SimulatorApiFlow.complete(
                requestId = instrumented.requestId,
                statusCode = null,
                error = throwable,
            )
            throw throwable
        }
    }
}
