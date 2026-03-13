# RANKING FORMULAS


This document records recommended formulas and calibration notes.
The formulas are deliberately simple enough for fast implementation and tuning.

## 1. Salience score

### Base formula
```text
score = bias
      + w_recency * recency_feature
      + w_salience * salience_feature
      + w_confidence * confidence_feature
      + w_utility * utility_feature
      + w_goal * goal_alignment_feature
      - w_noise * noise_feature
```

### Notes
- Start with clipped normalized features in [0,1].
- Use monotonic transforms to preserve interpretability.
- Calibrate by workload, not globally.
- Apply hard policy masks before soft ranking.

## 2. Confidence score

### Base formula
```text
score = bias
      + w_recency * recency_feature
      + w_salience * salience_feature
      + w_confidence * confidence_feature
      + w_utility * utility_feature
      + w_goal * goal_alignment_feature
      - w_noise * noise_feature
```

### Notes
- Start with clipped normalized features in [0,1].
- Use monotonic transforms to preserve interpretability.
- Calibrate by workload, not globally.
- Apply hard policy masks before soft ranking.

## 3. Utility estimate

### Base formula
```text
score = bias
      + w_recency * recency_feature
      + w_salience * salience_feature
      + w_confidence * confidence_feature
      + w_utility * utility_feature
      + w_goal * goal_alignment_feature
      - w_noise * noise_feature
```

### Notes
- Start with clipped normalized features in [0,1].
- Use monotonic transforms to preserve interpretability.
- Calibrate by workload, not globally.
- Apply hard policy masks before soft ranking.

## 4. Decay score

### Base formula
```text
score = bias
      + w_recency * recency_feature
      + w_salience * salience_feature
      + w_confidence * confidence_feature
      + w_utility * utility_feature
      + w_goal * goal_alignment_feature
      - w_noise * noise_feature
```

### Notes
- Start with clipped normalized features in [0,1].
- Use monotonic transforms to preserve interpretability.
- Calibrate by workload, not globally.
- Apply hard policy masks before soft ranking.

## 5. Promotion score

### Base formula
```text
score = bias
      + w_recency * recency_feature
      + w_salience * salience_feature
      + w_confidence * confidence_feature
      + w_utility * utility_feature
      + w_goal * goal_alignment_feature
      - w_noise * noise_feature
```

### Notes
- Start with clipped normalized features in [0,1].
- Use monotonic transforms to preserve interpretability.
- Calibrate by workload, not globally.
- Apply hard policy masks before soft ranking.

## 6. Demotion score

### Base formula
```text
score = bias
      + w_recency * recency_feature
      + w_salience * salience_feature
      + w_confidence * confidence_feature
      + w_utility * utility_feature
      + w_goal * goal_alignment_feature
      - w_noise * noise_feature
```

### Notes
- Start with clipped normalized features in [0,1].
- Use monotonic transforms to preserve interpretability.
- Calibrate by workload, not globally.
- Apply hard policy masks before soft ranking.

## 7. Conflict severity

### Base formula
```text
score = bias
      + w_recency * recency_feature
      + w_salience * salience_feature
      + w_confidence * confidence_feature
      + w_utility * utility_feature
      + w_goal * goal_alignment_feature
      - w_noise * noise_feature
```

### Notes
- Start with clipped normalized features in [0,1].
- Use monotonic transforms to preserve interpretability.
- Calibrate by workload, not globally.
- Apply hard policy masks before soft ranking.

## 8. Retrieval relevance

### Base formula
```text
score = bias
      + w_recency * recency_feature
      + w_salience * salience_feature
      + w_confidence * confidence_feature
      + w_utility * utility_feature
      + w_goal * goal_alignment_feature
      - w_noise * noise_feature
```

### Notes
- Start with clipped normalized features in [0,1].
- Use monotonic transforms to preserve interpretability.
- Calibrate by workload, not globally.
- Apply hard policy masks before soft ranking.

## 9. Novelty score

### Base formula
```text
score = bias
      + w_recency * recency_feature
      + w_salience * salience_feature
      + w_confidence * confidence_feature
      + w_utility * utility_feature
      + w_goal * goal_alignment_feature
      - w_noise * noise_feature
```

### Notes
- Start with clipped normalized features in [0,1].
- Use monotonic transforms to preserve interpretability.
- Calibrate by workload, not globally.
- Apply hard policy masks before soft ranking.

## 10. Compression value

### Base formula
```text
score = bias
      + w_recency * recency_feature
      + w_salience * salience_feature
      + w_confidence * confidence_feature
      + w_utility * utility_feature
      + w_goal * goal_alignment_feature
      - w_noise * noise_feature
```

### Notes
- Start with clipped normalized features in [0,1].
- Use monotonic transforms to preserve interpretability.
- Calibrate by workload, not globally.
- Apply hard policy masks before soft ranking.

