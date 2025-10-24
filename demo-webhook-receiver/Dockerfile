# Demo Webhook Receiver - For demonstration purposes only
# In production, customers deploy their own webhook endpoints

FROM python:3.11-slim

WORKDIR /app

# Install dependencies
RUN pip install --no-cache-dir flask

# Copy receiver script
COPY receiver.py .

# Expose port
EXPOSE 8000

# Run the receiver
CMD ["python", "receiver.py"]
