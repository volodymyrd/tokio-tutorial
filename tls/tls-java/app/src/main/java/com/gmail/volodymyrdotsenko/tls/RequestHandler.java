package com.gmail.volodymyrdotsenko.tls;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.List;

public final class RequestHandler implements Runnable {
    private static final Logger LOG = LoggerFactory.getLogger(RequestHandler.class);

    private final Service service;
    private final List<Request> requests;

    public RequestHandler(Service service, List<Request> requests) {
        this.service = service;
        this.requests = List.copyOf(requests);
    }

    @Override
    public void run() {
        LOG.info("Starting request handler with {} requests", requests.size());
        try {
            for (Request request : requests) {
                LOG.info("Sending request: {}", request);
                var response = service.get(request);
                LOG.info("Got response: {}", response);
            }
        } finally {
            service.clearContext();
        }
    }
}
