package com.gmail.volodymyrdotsenko.tls;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.Map;

public final class Service {
    private static final Logger LOG = LoggerFactory.getLogger(Service.class);

    private static final Map<String, String> CREDENTIALS = Map.of("user1", "pass1", "user2", "pass2");

    private static final ThreadLocal<String> LOGIN_CONTEXT = new ThreadLocal<>();

    public Response get(Request request) {
        LOG.info("Got request: {}", request);
        String loggedUser = LOGIN_CONTEXT.get();
        if (loggedUser != null) {
            LOG.info("User {} has been logged in already", loggedUser);
            return new Response(ResponseStatus.SUCCESS_ALREADY_LOGGED_IN);
        }
        if (!CREDENTIALS.containsKey(request.username())
                || !CREDENTIALS.get(request.username()).equals(request.password())) {
            return new Response(ResponseStatus.AUTH_ERROR);
        }

        LOGIN_CONTEXT.set(request.username());

        return new Response(ResponseStatus.SUCCESS);
    }

    public void clearContext() {
        LOGIN_CONTEXT.remove();
    }
}
